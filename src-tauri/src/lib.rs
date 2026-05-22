// 导入必要的 Rust 库
// serde: 用于序列化和反序列化，方便前端和后端传输数据
use serde::{Serialize, Deserialize};
// sysinfo: 用于获取系统信息和进程状态
use sysinfo::System;
// winapi: Windows API，用于操作进程优先级和 CPU 亲和性
use winapi::um::processthreadsapi::{OpenProcess, GetPriorityClass, SetPriorityClass};
use winapi::um::winbase::{GetProcessAffinityMask, SetProcessAffinityMask};
use winapi::um::winnt::{PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SET_INFORMATION};
use winapi::um::handleapi::CloseHandle;

// 进程状态枚举
// 1: 离线, 2: 在线, 3: 优化失败, 4: 已优化
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(try_from = "u32", into = "u32")]
enum ProcessState {
  Offline = 1,
  Online = 2,
  Failed = 3,
  Optimized = 4,
}

// 实现从 ProcessState 到 u32 的转换
impl From<ProcessState> for u32 {
  fn from(state: ProcessState) -> u32 {
    state as u32
  }
}

// 实现从 u32 到 ProcessState 的转换
impl TryFrom<u32> for ProcessState {
  type Error = String;

  fn try_from(value: u32) -> Result<Self, Self::Error> {
    match value {
      1 => Ok(ProcessState::Offline),
      2 => Ok(ProcessState::Online),
      3 => Ok(ProcessState::Failed),
      4 => Ok(ProcessState::Optimized),
      _ => Err(format!("Invalid ProcessState value: {}", value)),
    }
  }
}

// 定义进程状态结构体，供前端调用
// #[derive(Serialize)]: 自动实现序列化，方便发送到前端
// #[serde(rename_all = "camelCase")]: 将字段名转为驼峰式（符合 JavaScript 习惯）
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProcessStatus {
  name: String,           // 进程名
  state: ProcessState,    // 进程状态枚举
  updated_at: String,     // 更新时间
  hint: String,           // 提示信息
  pid: Option<u32>,       // 进程 ID
  priority: Option<String>,      // 优先级（中文显示）
  priority_key: Option<String>,   // 优先级（英文键，用于比对）
  affinity: Option<String>,      // CPU 亲和性
  core_count: Option<u32>,       // 使用核心数
}

// 定义优化配置结构体，供前端调用
// #[derive(Deserialize)]: 自动实现反序列化，方便接收前端数据
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct OptimizationConfig {
  processes: Vec<String>,        // 要优化的进程列表
  priority: String,               // 设置的优先级
  affinity: Vec<u32>,             // 设置的 CPU 亲和性核心
}

// 获取当前时间的字符串
fn get_current_time_str() -> String {
  let time = chrono::Local::now();
  format!("{}", time.format("%Y-%m-%d %H:%M:%S"))
}

// 将 Windows 优先级代码转为中文显示
fn get_priority_name(priority_class: u32) -> &'static str {
  match priority_class {
    0x00000040 => "低",
    0x00004000 => "低于正常",
    0x00000020 => "正常",
    0x00008000 => "高于正常",
    0x00000080 => "高",
    0x00000100 => "实时",
    _ => "未知",
  }
}

// 将优先级名称转为 Windows 代码
// 只支持英文格式（与前端配置保持一致）
// 支持的优先级: Normal, BelowNormal, Idle
fn get_priority_value(priority_name: &str) -> u32 {
  match priority_name {
    "Idle" | "idle" => 0x00000040,
    "BelowNormal" | "belownormal" => 0x00004000,
    "Normal" | "normal" => 0x00000020,
    _ => 0x00000020,
  }
}

// 将 Windows 优先级代码转为英文键（用于比对）
fn get_priority_key(priority_class: u32) -> &'static str {
  match priority_class {
    0x00000040 => "Idle",
    0x00004000 => "BelowNormal",
    0x00000020 => "Normal",
    0x00008000 => "AboveNormal",
    0x00000080 => "High",
    0x00000100 => "Realtime",
    _ => "Unknown",
  }
}

// 获取进程信息的函数
// 参数: pid 进程 ID
// 返回: (优先级, CPU 亲和性, 使用核心数)
fn get_process_info(pid: u32) -> (Option<String>, Option<String>, Option<String>, Option<u32>) {
  // 打开进程句柄，使用 PROCESS_QUERY_LIMITED_INFORMATION 权限（安全且够用）
  let handle = unsafe {
    OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid)
  };
  
  // 检查句柄是否有效
  if handle.is_null() {
    return (None, None, None, None);
  };
  
  // 声明变量用于接收 API 结果
  let mut process_affinity_mask: usize = 0;
  let mut system_affinity_mask: usize = 0;
  
  // 获取进程优先级
  let priority_result = unsafe {
    GetPriorityClass(handle)
  };
  
  // 获取进程 CPU 亲和性
  let affinity_result = unsafe {
    GetProcessAffinityMask(handle, &mut process_affinity_mask as *mut usize, &mut system_affinity_mask as *mut usize)
  };
  
  // 关闭句柄，释放资源
  unsafe {
    CloseHandle(handle);
  }

  // 获取中文优先级名称和英文优先级键
  let priority_cn = if priority_result != 0 {
    Some(get_priority_name(priority_result).to_string())
  } else {
    None
  };
  
  let priority_key = if priority_result != 0 {
    Some(get_priority_key(priority_result).to_string())
  } else {
    None
  };
  
  // 解析 CPU 亲和性掩码为可读的核心列表
  let affinity_str = if affinity_result != 0 && process_affinity_mask != 0 {
    let mut cores = Vec::new();
    for i in 0..64 {
      if ((process_affinity_mask as u64) & (1u64 << i)) != 0 {
        cores.push(i.to_string());
      }
    }
    Some(cores.join(", "))
  } else {
    None
  };
  
  // 计算使用的核心数
  let core_count = if process_affinity_mask != 0 {
    Some((process_affinity_mask as u64).count_ones())
  } else {
    None
  };
  
  (priority_cn, priority_key, affinity_str, core_count)
}

// 检查进程是否在运行
// 参数: sys System 实例, name 进程名
// 返回: Some((pid, 中文优先级, 英文优先级键, CPU 亲和性, 使用核心数)) 或 None
fn check_process_running(sys: &System, name: &str) -> Option<(u32, Option<String>, Option<String>, Option<String>, Option<u32>)> {
  // 遍历系统中所有进程
  for process in sys.processes_by_name(name) {
    let pid = process.pid().as_u32();
    
    // 获取该进程的详细信息
    let (priority_cn, priority_key, affinity, core_count) = get_process_info(pid);
    
    return Some((pid, priority_cn, priority_key, affinity, core_count));
  }
  None
}

// 优化进程（设置优先级和 CPU 亲和性）
// 参数: pid 进程 ID, priority_name 优先级名称, affinity_cores CPU 核心列表
// 返回: 是否成功
fn optimize_process(pid: u32, priority_name: &str, affinity_cores: &[u32]) -> bool {
  // 打开进程句柄，需要 PROCESS_SET_INFORMATION 权限来修改进程设置
  let handle = unsafe {
    OpenProcess(PROCESS_SET_INFORMATION | PROCESS_QUERY_LIMITED_INFORMATION, 0, pid)
  };
  
  if handle.is_null() {
    return false;
  }
  
  let mut success = true;
  
  // 设置进程优先级
  let priority_value = get_priority_value(priority_name);
  
  let priority_result = unsafe {
    SetPriorityClass(handle, priority_value)
  };
  
  if priority_result == 0 {
    success = false;
  }
  
  // 设置进程 CPU 亲和性
  if !affinity_cores.is_empty() {
    let mut affinity_mask: u32 = 0;
    for &core in affinity_cores {
      if core < 32 {
        affinity_mask |= 1u32 << core;
      }
    }
    
    if affinity_mask != 0 {
      let affinity_result = unsafe {
        SetProcessAffinityMask(handle, affinity_mask)
      };
      
      if affinity_result == 0 {
        success = false;
      }
    }
  }
  
  // 关闭句柄
  unsafe {
    CloseHandle(handle);
  }
  
  success
}

// =========================================================
// Tauri 命令：供前端调用的函数
// 注意：#[tauri::command] 标记表示这是一个 Tauri 命令
// 前端可以通过 invoke('get_process_status') 调用
// =========================================================

// 获取所有进程状态的 Tauri 命令
#[tauri::command]
fn get_process_status() -> Vec<ProcessStatus> {
  // 创建 System 实例并刷新进程列表
  let mut sys = System::new_all();
  sys.refresh_all();
  
  let mut results = Vec::new();
  // 要检测的进程列表
  let processes_to_check = ["SGuardSvc64.exe", "SGuard64.exe", "ACE-Tray.exe"];
  
  // 遍历检查每个进程
  for &proc_name in &processes_to_check {
    if let Some((pid, priority_cn, priority_key, affinity, core_count)) = check_process_running(&sys, proc_name) {
      let mut hint = format!("发现进程在运行 (PID: {})", pid);
      if let Some(p) = &priority_cn {
        hint.push_str(&format!("，优先级: {}", p));
      }
      if let Some(a) = &affinity {
        hint.push_str(&format!("，CPU: {}", a));
      }
      
      // 对于 get_process_status，我们只检测进程是否存在和获取信息
      // 由于没有优化配置，我们使用 Online 状态表示进程存在
      // 优化操作由 optimize_processes 处理
      results.push(ProcessStatus {
        name: proc_name.to_string(),
        state: ProcessState::Online,
        updated_at: get_current_time_str(),
        hint,
        pid: Some(pid),
        priority: priority_cn,
        priority_key: priority_key,
        affinity,
        core_count,
      });
    } else {
      results.push(ProcessStatus {
        name: proc_name.to_string(),
        state: ProcessState::Offline,
        updated_at: get_current_time_str(),
        hint: "没有发现它在运行哦".to_string(),
        pid: None,
        priority: None,
        priority_key: None,
        affinity: None,
        core_count: None,
      });
    }
  }
  
  results
}

// 优化进程的 Tauri 命令
#[tauri::command]
fn optimize_processes(config: OptimizationConfig) -> Vec<ProcessStatus> {
  let mut sys = System::new_all();
  sys.refresh_all();

  let mut results = Vec::new();
  let mut actual_optimize_count = 0;

  for proc_name in &config.processes {
    // 首先检测进程是否在运行
    if let Some((pid, current_priority_cn, current_priority_key, current_affinity, current_core_count)) = check_process_running(&sys, proc_name) {
      // 检查进程是否已经符合优化配置（不依赖前端状态）
      let is_priority_optimized = current_priority_key
        .as_ref()
        .map(|pk| pk == &config.priority)
        .unwrap_or(false);

      let is_affinity_optimized = if !config.affinity.is_empty() {
        if let Some(aff_str) = &current_affinity {
          let current_cores: Vec<u32> = aff_str
            .split(", ")
            .filter_map(|s| s.parse().ok())
            .collect();
          let mut target_cores = config.affinity.clone();
          let mut current_cores_sorted = current_cores.clone();
          target_cores.sort();
          current_cores_sorted.sort();
          target_cores == current_cores_sorted
        } else {
          false
        }
      } else {
        true
      };

      if is_priority_optimized && is_affinity_optimized {
        // 进程已经符合优化配置，跳过优化
        let mut hint = format!("已优化 (PID: {})", pid);
        if let Some(p) = &current_priority_cn {
          hint.push_str(&format!("，优先级: {}", p));
        }
        if let Some(a) = &current_affinity {
          hint.push_str(&format!("，CPU: {}", a));
        }

        results.push(ProcessStatus {
          name: proc_name.to_string(),
          state: ProcessState::Optimized,
          updated_at: get_current_time_str(),
          hint,
          pid: Some(pid),
          priority: current_priority_cn,
          priority_key: current_priority_key,
          affinity: current_affinity,
          core_count: current_core_count,
        });
        continue;
      }

      // 进程不符合优化配置，需要进行优化
      actual_optimize_count += 1;
      let optimize_success = optimize_process(pid, &config.priority, &config.affinity);

      // 获取优化后的最新信息
      let (priority_cn, priority_key, affinity, core_count) = get_process_info(pid);

      let mut hint = format!("已优化 (PID: {})", pid);
      if let Some(p) = &priority_cn {
        hint.push_str(&format!("，优先级: {}", p));
      }
      if let Some(a) = &affinity {
        hint.push_str(&format!("，CPU: {}", a));
      }

      let final_state = if optimize_success {
        ProcessState::Optimized
      } else {
        ProcessState::Failed
      };

      results.push(ProcessStatus {
        name: proc_name.to_string(),
        state: final_state,
        updated_at: get_current_time_str(),
        hint,
        pid: Some(pid),
        priority: priority_cn,
        priority_key: priority_key,
        affinity,
        core_count,
      });
    } else {
      // 进程不在，状态变成离线
      results.push(ProcessStatus {
        name: proc_name.to_string(),
        state: ProcessState::Offline,
        updated_at: get_current_time_str(),
        hint: "没有发现它在运行哦".to_string(),
        pid: None,
        priority: None,
        priority_key: None,
        affinity: None,
        core_count: None,
      });
    }
  }
  
  // 只在真正优化了进程时打印
  if actual_optimize_count > 0 {
    println!("[ACE Helper] Optimized {} processes", actual_optimize_count);
  }
  
  results
}

// =========================================================
// Tauri 应用入口函数
// =========================================================
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    // 注册 Tauri 命令，前端才能调用
    // 注意：必须在这里列出所有 #[tauri::command] 标记的函数
    .invoke_handler(tauri::generate_handler![get_process_status, optimize_processes])
    // 注册自动启动插件
    .plugin(tauri_plugin_autostart::init(
      tauri_plugin_autostart::MacosLauncher::LaunchAgent,
      None,
    ))
    .setup(|app| {
      // 调试模式下启用日志插件
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
