// 导入必要的 Rust 库
// serde: 用于序列化和反序列化，方便前端和后端传输数据
use serde::{Serialize, Deserialize};
// sysinfo: 用于获取系统信息和进程状态
use sysinfo::System;
// winapi: Windows API，用于操作进程优先级和 CPU 亲和性
use winapi::um::processthreadsapi::{OpenProcess, GetPriorityClass, SetPriorityClass, GetCurrentProcess, OpenProcessToken};
use winapi::um::winbase::{GetProcessAffinityMask, SetProcessAffinityMask, LookupPrivilegeValueW};
use winapi::um::winnt::{PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SET_INFORMATION, PROCESS_QUERY_INFORMATION, TOKEN_ADJUST_PRIVILEGES, TOKEN_QUERY, SE_PRIVILEGE_ENABLED};
use winapi::shared::ntdef::NTSTATUS;
use winapi::um::handleapi::CloseHandle;
use winapi::um::securitybaseapi::AdjustTokenPrivileges;
// Tauri 系统托盘
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::Manager;
use tauri::Emitter;
use tauri_plugin_notification::NotificationExt;
use std::sync::{Mutex, Once};
use std::ptr;
use std::process::Command;
use std::os::windows::process::CommandExt;

// Windows 效率模式相关常量
const PROCESS_INFORMATION_CLASS_POWER_THROTTLING: u32 = 4;
const PROCESS_POWER_THROTTLING_STATE_VERSION: u32 = 1;
const PROCESS_POWER_THROTTLING_EXECUTION_SPEED: u32 = 1;

// Windows 版本检测结构体
#[repr(C)]
#[allow(non_snake_case, dead_code)]
#[repr(C)]
struct OSVERSIONINFOEXW {
    dwOSVersionInfoSize: u32,
    dwMajorVersion: u32,
    dwMinorVersion: u32,
    dwBuildNumber: u32,
    dwPlatformId: u32,
    szCSDVersion: [u16; 128],
    wServicePackMajor: u16,
    wServicePackMinor: u16,
    wSuiteMask: u16,
    wProductType: u8,
    wReserved: u8,
}

// Windows 版本检测
// 返回 (主版本, 次版本, 构建号)
fn get_windows_version() -> (u32, u32, u32) {
  unsafe {
    // 使用 ntdll 的 RtlGetVersion，比 GetVersionEx 更可靠
    type RtlGetVersionFn = unsafe extern "system" fn(*mut OSVERSIONINFOEXW) -> i32;
    
    let ntdll = winapi::um::libloaderapi::GetModuleHandleA(b"ntdll.dll\0".as_ptr() as *const i8);
    if ntdll.is_null() {
      return (10, 0, 19041); // 默认 Windows 10
    }
    
    let rtl_get_version: RtlGetVersionFn = std::mem::transmute(
      winapi::um::libloaderapi::GetProcAddress(ntdll, b"RtlGetVersion\0".as_ptr() as *const i8)
    );
    
    if rtl_get_version as usize == 0 {
      return (10, 0, 19041); // 默认 Windows 10
    }
    
    let mut osvi: OSVERSIONINFOEXW = std::mem::zeroed();
    osvi.dwOSVersionInfoSize = std::mem::size_of::<OSVERSIONINFOEXW>() as u32;
    
    let result = rtl_get_version(&mut osvi);
    
    if result == 0 {
      (osvi.dwMajorVersion, osvi.dwMinorVersion, osvi.dwBuildNumber)
    } else {
      // 默认返回 Windows 10
      (10, 0, 19041)
    }
  }
}

// 检查是否支持效率模式（Windows 11 或 Windows 10 21H2+ 且需要特定构建版本）
fn is_efficiency_mode_supported() -> bool {
  let (major, _minor, build) = get_windows_version();
  // Windows 11 (主版本 10，构建号 >= 22000)
  // 或 Windows 10 21H2+ (构建号 >= 19044)
  (major >= 10 && build >= 22000) || (major >= 10 && build >= 19044)
}

#[repr(C)]
struct PROCESS_POWER_THROTTLING_INFORMATION {
    version: u32,
    control_mask: u32,
    state_mask: u32,
}

// 使用 SetProcessInformation 和 GetProcessInformation
extern "system" {
  fn SetProcessInformation(
    ProcessHandle: *mut winapi::ctypes::c_void,
    ProcessInformationClass: u32,
    ProcessInformation: *const winapi::ctypes::c_void,
    ProcessInformationLength: u32,
  ) -> i32;
  
  fn GetProcessInformation(
    ProcessHandle: *mut winapi::ctypes::c_void,
    ProcessInformationClass: u32,
    ProcessInformation: *mut winapi::ctypes::c_void,
    ProcessInformationLength: u32,
  ) -> i32;
}

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
  io_priority: Option<String>,   // 磁盘 I/O 优先级（中文显示）
  io_priority_key: Option<String>, // 磁盘 I/O 优先级（英文键，用于比对）
  efficiency_mode: Option<bool>, // 效率模式
}

// 定义优化配置结构体，供前端调用
// #[derive(Deserialize)]: 自动实现反序列化，方便接收前端数据
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct OptimizationConfig {
  processes: Vec<String>,        // 要优化的进程列表
  priority: String,               // 设置的优先级
  affinity: Vec<u32>,             // 设置的 CPU 亲和性核心
  io_priority: String,            // 设置的磁盘 I/O 优先级
  efficiency_mode: bool,          // 是否启用效率模式
}

// 获取当前时间的字符串
fn get_current_time_str() -> String {
  let time = chrono::Local::now();
  format!("{}", time.format("%Y-%m-%d %H:%M:%S"))
}

// 全局 Once 确保 SeDebugPrivilege 只启用一次
static SE_DEBUG_ONCE: Once = Once::new();

// 启用 SeDebugPrivilege（允许访问受保护进程）
fn enable_se_debug_privilege() -> bool {
    unsafe {
        let mut token_handle: *mut winapi::ctypes::c_void = ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY, &mut token_handle) == 0 {
            return false;
        }

        let mut luid: winapi::um::winnt::LUID = std::mem::zeroed();
        let privilege_name: Vec<u16> = "SeDebugPrivilege\0".encode_utf16().collect();
        if LookupPrivilegeValueW(ptr::null(), privilege_name.as_ptr(), &mut luid) == 0 {
            CloseHandle(token_handle);
            return false;
        }

        let mut tp: winapi::um::winnt::TOKEN_PRIVILEGES = std::mem::zeroed();
        tp.PrivilegeCount = 1;
        tp.Privileges[0].Luid = luid;
        tp.Privileges[0].Attributes = SE_PRIVILEGE_ENABLED;

        let result = AdjustTokenPrivileges(
            token_handle,
            0,
            &mut tp,
            std::mem::size_of::<winapi::um::winnt::TOKEN_PRIVILEGES>() as u32,
            ptr::null_mut(),
            ptr::null_mut(),
        );

        CloseHandle(token_handle);
        result != 0
    }
}

// 确保 SeDebugPrivilege 已启用（全局仅执行一次）
fn ensure_se_debug_privilege() {
    SE_DEBUG_ONCE.call_once(|| {
        if enable_se_debug_privilege() {
            println!("[ACE Helper] SeDebugPrivilege enabled");
        } else {
            eprintln!("[ACE Helper] Warning: failed to enable SeDebugPrivilege");
        }
    });
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

// 将磁盘 I/O 优先级名称转为 Windows 代码
fn get_io_priority_value(io_priority_name: &str) -> u32 {
  match io_priority_name {
    "VeryLow" | "verylow" => 0,  // IoPriorityVeryLow
    "Low" | "low" => 1,          // IoPriorityLow
    "Normal" | "normal" => 2,    // IoPriorityNormal
    _ => 0,  // 默认 VeryLow
  }
}

// 将 Windows 磁盘 I/O 优先级代码转为中文显示
fn get_io_priority_name(io_priority: u32) -> &'static str {
  match io_priority {
    0 => "非常低",
    1 => "低",
    2 => "正常",
    _ => "未知",
  }
}

// 将 Windows 磁盘 I/O 优先级代码转为英文键（用于比对）
fn get_io_priority_key(io_priority: u32) -> &'static str {
  match io_priority {
    0 => "VeryLow",
    1 => "Low",
    2 => "Normal",
    _ => "Unknown",
  }
}

// Windows I/O 优先级常量
const PROCESS_IO_PRIORITY: u32 = 33;

// 使用 NtQueryInformationProcess 获取 I/O 优先级
extern "system" {
  fn NtQueryInformationProcess(
    ProcessHandle: *mut winapi::ctypes::c_void,
    ProcessInformationClass: u32,
    ProcessInformation: *mut winapi::ctypes::c_void,
    ProcessInformationLength: u32,
    ReturnLength: *mut u32,
  ) -> NTSTATUS;
  
  fn NtSetInformationProcess(
    ProcessHandle: *mut winapi::ctypes::c_void,
    ProcessInformationClass: u32,
    ProcessInformation: *const winapi::ctypes::c_void,
    ProcessInformationLength: u32,
  ) -> NTSTATUS;
}

// 获取进程的 I/O 优先级
fn get_process_io_priority(handle: *mut winapi::ctypes::c_void) -> Option<u32> {
  unsafe {
    let mut io_priority: u32 = 0;
    let status = NtQueryInformationProcess(
      handle,
      PROCESS_IO_PRIORITY,
      &mut io_priority as *mut u32 as *mut winapi::ctypes::c_void,
      std::mem::size_of::<u32>() as u32,
      std::ptr::null_mut(),
    );
    if status >= 0 {
      Some(io_priority)
    } else {
      None
    }
  }
}

// 设置进程的 I/O 优先级
fn set_process_io_priority(handle: *mut winapi::ctypes::c_void, io_priority: u32) -> bool {
  unsafe {
    let status = NtSetInformationProcess(
      handle,
      PROCESS_IO_PRIORITY,
      &io_priority as *const u32 as *const winapi::ctypes::c_void,
      std::mem::size_of::<u32>() as u32,
    );
    status >= 0
  }
}

// 获取进程的效率模式状态
// 在 Windows 10 上返回 None，因为该功能不可用
fn get_process_efficiency_mode(handle: *mut winapi::ctypes::c_void) -> Option<bool> {
  // Windows 10 不支持效率模式，直接返回 None
  if !is_efficiency_mode_supported() {
    return None;
  }
  
  unsafe {
    let mut state: PROCESS_POWER_THROTTLING_INFORMATION = std::mem::zeroed();
    state.version = PROCESS_POWER_THROTTLING_STATE_VERSION;
    
    let result = GetProcessInformation(
      handle,
      PROCESS_INFORMATION_CLASS_POWER_THROTTLING,
      &mut state as *mut PROCESS_POWER_THROTTLING_INFORMATION as *mut winapi::ctypes::c_void,
      std::mem::size_of::<PROCESS_POWER_THROTTLING_INFORMATION>() as u32,
    );
    
    if result != 0 {
      if (state.control_mask & PROCESS_POWER_THROTTLING_EXECUTION_SPEED) != 0 {
        Some((state.state_mask & PROCESS_POWER_THROTTLING_EXECUTION_SPEED) != 0)
      } else {
        Some(false)
      }
    } else {
      None
    }
  }
}

// 设置进程的效率模式
// 在 Windows 10 上会自动跳过，因为该功能需要 Windows 11+
fn set_process_efficiency_mode(handle: *mut winapi::ctypes::c_void, enable: bool) -> bool {
  // Windows 10 不支持效率模式，直接返回 true 表示跳过
  if !is_efficiency_mode_supported() {
    println!("[ACE Helper] 效率模式不支持当前 Windows 版本，已跳过");
    return true;
  }
  
  unsafe {
    let mut state: PROCESS_POWER_THROTTLING_INFORMATION = std::mem::zeroed();
    state.version = PROCESS_POWER_THROTTLING_STATE_VERSION;
    state.control_mask = PROCESS_POWER_THROTTLING_EXECUTION_SPEED;
    state.state_mask = if enable { PROCESS_POWER_THROTTLING_EXECUTION_SPEED } else { 0 };
    
    let result = SetProcessInformation(
      handle,
      PROCESS_INFORMATION_CLASS_POWER_THROTTLING,
      &state as *const PROCESS_POWER_THROTTLING_INFORMATION as *const winapi::ctypes::c_void,
      std::mem::size_of::<PROCESS_POWER_THROTTLING_INFORMATION>() as u32,
    );
    
    result != 0
  }
}

// PowerShell 回退方案：通过 PowerShell 获取进程信息
// 适用于 Win32 OpenProcess 被内核级保护拦截的进程
fn get_process_info_via_powershell(pid: u32) -> (Option<String>, Option<String>, Option<String>, Option<u32>, Option<String>, Option<String>, Option<bool>) {
    let ps_script = format!(
        "$p=Get-Process -Id {} -ErrorAction Stop; \
         $p | Select-Object PriorityClass, ProcessorAffinity | ConvertTo-Json",
        pid
    );

    let output = Command::new("powershell")
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .args(["-NoProfile", "-Command", &ps_script])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let json_str = stdout.trim();
            
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(json_str) {
                let priority_cn = json.get("PriorityClass")
                    .and_then(|v| v.as_str())
                    .map(|s| match s {
                        "Idle" => "低",
                        "BelowNormal" => "低于正常",
                        "Normal" => "正常",
                        "AboveNormal" => "高于正常",
                        "High" => "高",
                        "Realtime" => "实时",
                        _ => "未知",
                    })
                    .map(|s| s.to_string());
                
                let priority_key = json.get("PriorityClass")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                
                let affinity_num = json.get("ProcessorAffinity")
                    .and_then(|v| v.as_u64());
                
                let (affinity_str, core_count) = if let Some(mask) = affinity_num {
                    let mut cores = Vec::new();
                    for i in 0..64 {
                        if (mask & (1u64 << i)) != 0 {
                            cores.push(i.to_string());
                        }
                    }
                    let count = mask.count_ones();
                    (Some(cores.join(", ")), Some(count))
                } else {
                    (None, None)
                };
                
                return (priority_cn, priority_key, affinity_str, core_count, None, None, None);
            }
            
            (None, None, None, None, None, None, None)
        }
        _ => (None, None, None, None, None, None, None)
    }
}

// 获取进程信息的函数
// 参数: pid 进程 ID
// 返回: (优先级, 优先级键, CPU 亲和性, 使用核心数, I/O 优先级, I/O 优先级键, 效率模式)
fn get_process_info(pid: u32) -> (Option<String>, Option<String>, Option<String>, Option<u32>, Option<String>, Option<String>, Option<bool>) {
  // 确保已启用 SeDebugPrivilege
  ensure_se_debug_privilege();
  
  // 尝试使用更高权限打开进程
  let handle = unsafe {
    // 先尝试 PROCESS_QUERY_INFORMATION（更高权限）
    let h = OpenProcess(PROCESS_QUERY_INFORMATION, 0, pid);
    if h.is_null() {
      // 降级到 PROCESS_QUERY_LIMITED_INFORMATION
      OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid)
    } else {
      h
    }
  };
  
  // 检查句柄是否有效
  if handle.is_null() {
    // Win32 API 失败，尝试使用 PowerShell 获取进程信息
    return get_process_info_via_powershell(pid);
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
  
  // 获取 I/O 优先级（在关闭句柄之前）
  let io_priority = get_process_io_priority(handle);
  let io_priority_cn = io_priority.map(|p| get_io_priority_name(p).to_string());
  let io_priority_key = io_priority.map(|p| get_io_priority_key(p).to_string());
  
  // 获取效率模式状态
  let efficiency_mode = get_process_efficiency_mode(handle);
  
  // 关闭句柄，释放资源
  unsafe {
    CloseHandle(handle);
  }
  
  (priority_cn, priority_key, affinity_str, core_count, io_priority_cn, io_priority_key, efficiency_mode)
}

// 检查进程是否在运行
// 参数: sys System 实例, name 进程名
// 返回: Some((pid, 中文优先级, 英文优先级键, CPU 亲和性, 使用核心数, I/O 优先级, I/O 优先级键, 效率模式)) 或 None
fn check_process_running(sys: &System, name: &str) -> Option<(u32, Option<String>, Option<String>, Option<String>, Option<u32>, Option<String>, Option<String>, Option<bool>)> {
  // 将目标进程名转为小写用于不区分大小写的比较
  let target_name_lower = name.to_lowercase();
  
  // 遍历系统中所有进程
  for (pid, process) in sys.processes() {
    let process_name = process.name().to_string();
    
    // 不区分大小写比较进程名
    if process_name.to_lowercase() == target_name_lower {
      let pid_u32 = pid.as_u32();
      
      // 获取该进程的详细信息
      let (priority_cn, priority_key, affinity, core_count, io_priority_cn, io_priority_key, efficiency_mode) = get_process_info(pid_u32);
      
      return Some((pid_u32, priority_cn, priority_key, affinity, core_count, io_priority_cn, io_priority_key, efficiency_mode));
    }
  }
  None
}

// 优化进程（设置优先级、CPU 亲和性、I/O 优先级和效率模式）
// 参数: pid 进程 ID, priority_name 优先级名称, affinity_cores CPU 核心列表, io_priority_name I/O 优先级名称, efficiency_mode 效率模式
// 返回: 是否成功
fn optimize_process(pid: u32, priority_name: &str, affinity_cores: &[u32], io_priority_name: &str, efficiency_mode: bool) -> (bool, String) {
  // 确保已启用 SeDebugPrivilege
  ensure_se_debug_privilege();
  
  // 尝试使用更高权限打开进程
  let handle = unsafe {
    // 先尝试 PROCESS_SET_INFORMATION | PROCESS_QUERY_INFORMATION
    let h = OpenProcess(PROCESS_SET_INFORMATION | PROCESS_QUERY_INFORMATION, 0, pid);
    if h.is_null() {
      // 降级到较低权限
      OpenProcess(PROCESS_SET_INFORMATION | PROCESS_QUERY_LIMITED_INFORMATION, 0, pid)
    } else {
      h
    }
  };
  
  if handle.is_null() {
    // Win32 OpenProcess 失败（可能受内核保护），回退到 PowerShell Get-Process
    return optimize_process_via_powershell(pid, priority_name, affinity_cores, io_priority_name);
  }
  
  let mut success = true;
  let mut fail_reason = String::new();
  
  let priority_value = get_priority_value(priority_name);
  
  let priority_result = unsafe {
    SetPriorityClass(handle, priority_value)
  };
  
  if priority_result == 0 {
    success = false;
    if fail_reason.is_empty() {
      fail_reason = "设置优先级被拒绝".to_string();
    }
  }
  
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
        if fail_reason.is_empty() {
          fail_reason = "设置CPU亲和性被拒绝".to_string();
        }
      }
    }
  }
  
  // 设置 I/O 优先级
  let io_priority_value = get_io_priority_value(io_priority_name);
  if !set_process_io_priority(handle, io_priority_value) {
    success = false;
    if fail_reason.is_empty() {
      fail_reason = "设置磁盘I/O优先级被拒绝".to_string();
    }
  }
  
  // 设置效率模式（失败时不阻断整体优化）
  set_process_efficiency_mode(handle, efficiency_mode);
  
  // 关闭句柄
  unsafe {
    CloseHandle(handle);
  }
  
  (success, fail_reason)
}

// PowerShell 回退方案：适用于 Win32 OpenProcess 被内核级保护拦截的进程
// 通过 PowerShell 的 Get-Process 操作进程（部分反作弊驱动对微软签名进程放行）
fn optimize_process_via_powershell(pid: u32, priority_name: &str, affinity_cores: &[u32], _io_priority_name: &str) -> (bool, String) {
    // 将优先级名映射到 PowerShell 可识别的格式
    let pri = match priority_name.to_lowercase().as_str() {
        "idle" => "Idle",
        "belownormal" => "BelowNormal",
        "normal" => "Normal",
        _ => "Normal",
    };

    // 计算 CPU 亲和性掩码
    let mut affinity_mask: u64 = 0;
    for &core in affinity_cores {
        if core < 64 {
            affinity_mask |= 1u64 << core;
        }
    }

    let affinity_str = if affinity_mask > 0 {
        format!("; $p.ProcessorAffinity={}", affinity_mask)
    } else {
        String::new()
    };

    // 注意：PowerShell 无法直接设置 I/O 优先级，需要通过 WMI 或 P/Invoke
    // 这里我们只设置常规优先级和亲和性，I/O 优先级在 Win32 API 方案中处理
    let ps_script = format!(
        "$p=Get-Process -Id {} -ErrorAction Stop; \
         $p.PriorityClass='{}'{}; \
         exit 0",
        pid, pri, affinity_str
    );

    let output = Command::new("powershell")
        .creation_flags(0x08000000) // CREATE_NO_WINDOW，无窗口闪现
        .args(["-NoProfile", "-Command", &ps_script])
        .output();

    match output {
        Ok(out) if out.status.success() => (true, String::new()),
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            let err_msg = stderr.trim().to_string();
            if err_msg.is_empty() {
                (false, "PowerShell 操作被拒绝（即使 Get-Process 也无法访问该进程）".to_string())
            } else {
                (false, format!("PowerShell 拒绝: {}", err_msg))
            }
        }
        Err(e) => (false, format!("无法启动 PowerShell: {}", e)),
    }
}

// =========================================================
// Tauri 命令：供前端调用的函数
// 注意：#[tauri::command] 标记表示这是一个 Tauri 命令
// 前端可以通过 invoke('get_process_status') 调用
// =========================================================

struct TrayState {
  minimize_to_tray: Mutex<bool>,
}

#[tauri::command]
fn set_minimize_to_tray(state: tauri::State<TrayState>, value: bool) {
  *state.minimize_to_tray.lock().unwrap() = value;
}

#[tauri::command]
fn send_notification(app: tauri::AppHandle, title: String, body: String) {
  let result = app
    .notification()
    .builder()
    .title(&title)
    .body(&body)
    .show();
  if let Err(e) = result {
    eprintln!("[ACE Helper] 通知发送失败: {:?}", e);
  } else {
    println!("[ACE Helper] 通知已发送: {}", title);
  }
}

// 获取 Windows 显示名称
fn get_windows_display_name(major: u32, _minor: u32, build: u32) -> String {
  if major >= 10 && build >= 22000 {
    "Windows 11".to_string()
  } else if major >= 10 && build >= 10240 {
    "Windows 10".to_string()
  } else if major == 6 && _minor == 3 {
    "Windows 8.1".to_string()
  } else if major == 6 && _minor == 2 {
    "Windows 8".to_string()
  } else if major == 6 && _minor == 1 {
    "Windows 7".to_string()
  } else {
    format!("Windows {}.{} (Build {})", major, _minor, build)
  }
}

// 获取系统信息的 Tauri 命令
// 返回系统版本信息和是否支持效率模式
#[tauri::command]
fn get_system_info() -> serde_json::Value {
  let (major, minor, build) = get_windows_version();
  let efficiency_supported = is_efficiency_mode_supported();
  let display_name = get_windows_display_name(major, minor, build);
  
  serde_json::json!({
    "platform": "windows",
    "version": {
      "major": major,
      "minor": minor,
      "build": build,
      "displayName": display_name
    },
    "efficiencyModeSupported": efficiency_supported,
    "efficiencyModeNote": if efficiency_supported { None } else { Some("效率模式需要 Windows 11 或 Windows 10 21H2+") }
  })
}

// 获取进程状态的 Tauri 命令
// 参数: processes 要检查的进程名列表, config 优化配置（可选）
#[tauri::command]
fn get_process_status(processes: Vec<String>, config: Option<OptimizationConfig>) -> Vec<ProcessStatus> {
  // 创建 System 实例并刷新进程列表
  let mut sys = System::new_all();
  sys.refresh_all();
  
  let mut results = Vec::new();
  
  // 遍历检查每个进程
  for proc_name in &processes {
    if let Some((pid, priority_cn, priority_key, affinity, core_count, io_priority_cn, io_priority_key, efficiency_mode)) = check_process_running(&sys, proc_name) {
      // 判断进程是否已优化（如果提供了配置）
      let is_optimized = if let Some(ref cfg) = config {
        let is_priority_match = priority_key
          .as_ref()
          .map(|pk| pk == &cfg.priority)
          .unwrap_or(false);
        
        let is_affinity_match = if !cfg.affinity.is_empty() {
          if let Some(ref aff_str) = affinity {
            let current_cores: Vec<u32> = aff_str
              .split(", ")
              .filter_map(|s| s.parse().ok())
              .collect();
            let mut target_cores = cfg.affinity.clone();
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
        
        let is_io_priority_match = io_priority_key
          .as_ref()
          .map(|io| io == &cfg.io_priority)
          .unwrap_or(false);
        
        let is_efficiency_mode_match = efficiency_mode
          .map(|em| em == cfg.efficiency_mode)
          .unwrap_or(false);
        
        is_priority_match && is_affinity_match && is_io_priority_match && is_efficiency_mode_match
      } else {
        false
      };
      
      let state = if is_optimized {
        ProcessState::Optimized
      } else {
        ProcessState::Online
      };
      
      let mut hint = if is_optimized {
        format!("已优化 (PID: {})", pid)
      } else {
        format!("发现进程在运行 (PID: {})", pid)
      };
      
      if let Some(p) = &priority_cn {
        hint.push_str(&format!("，优先级: {}", p));
      }
      if let Some(a) = &affinity {
        hint.push_str(&format!("，CPU: {}", a));
      }
      if let Some(io) = &io_priority_cn {
        hint.push_str(&format!("，磁盘I/O: {}", io));
      }
      if let Some(em) = efficiency_mode {
        hint.push_str(&format!("，效率模式: {}", if em { "已开启" } else { "未开启" }));
      }
      
      results.push(ProcessStatus {
        name: proc_name.to_string(),
        state,
        updated_at: get_current_time_str(),
        hint,
        pid: Some(pid),
        priority: priority_cn,
        priority_key: priority_key,
        affinity,
        core_count,
        io_priority: io_priority_cn,
        io_priority_key: io_priority_key,
        efficiency_mode,
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
        io_priority: None,
        io_priority_key: None,
        efficiency_mode: None,
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
    if let Some((pid, current_priority_cn, current_priority_key, current_affinity, current_core_count, current_io_priority_cn, current_io_priority_key, current_efficiency_mode)) = check_process_running(&sys, proc_name) {
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
      
      let is_io_priority_optimized = current_io_priority_key
        .as_ref()
        .map(|io| io == &config.io_priority)
        .unwrap_or(false);

      let is_efficiency_mode_optimized = current_efficiency_mode
        .map(|em| em == config.efficiency_mode)
        .unwrap_or(false);

      if is_priority_optimized && is_affinity_optimized && is_io_priority_optimized && is_efficiency_mode_optimized {
        // 进程已经符合优化配置，跳过优化
        let mut hint = format!("已优化 (PID: {})", pid);
        if let Some(p) = &current_priority_cn {
          hint.push_str(&format!("，优先级: {}", p));
        }
        if let Some(a) = &current_affinity {
          hint.push_str(&format!("，CPU: {}", a));
        }
        if let Some(io) = &current_io_priority_cn {
          hint.push_str(&format!("，磁盘I/O: {}", io));
        }
        if let Some(em) = current_efficiency_mode {
          hint.push_str(&format!("，效率模式: {}", if em { "已开启" } else { "未开启" }));
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
          io_priority: current_io_priority_cn,
          io_priority_key: current_io_priority_key,
          efficiency_mode: current_efficiency_mode,
        });
        continue;
      }

      // 进程不符合优化配置，需要进行优化
      actual_optimize_count += 1;
      let (optimize_success, fail_reason) = optimize_process(pid, &config.priority, &config.affinity, &config.io_priority, config.efficiency_mode);

      // 获取优化后的最新信息
      let (priority_cn, priority_key, affinity, core_count, io_priority_cn, io_priority_key, efficiency_mode) = get_process_info(pid);

      let final_state = if optimize_success {
        ProcessState::Optimized
      } else {
        ProcessState::Failed
      };

      let hint = if optimize_success {
        let mut h = format!("已优化 (PID: {})", pid);
        if let Some(p) = &priority_cn {
          h.push_str(&format!("，优先级: {}", p));
        }
        if let Some(a) = &affinity {
          h.push_str(&format!("，CPU: {}", a));
        }
        if let Some(io) = &io_priority_cn {
          h.push_str(&format!("，磁盘I/O: {}", io));
        }
        if let Some(em) = efficiency_mode {
          h.push_str(&format!("，效率模式: {}", if em { "已开启" } else { "未开启" }));
        }
        h
      } else {
        format!("优化失败 (PID: {}) — {}", pid, fail_reason)
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
        io_priority: io_priority_cn,
        io_priority_key: io_priority_key,
        efficiency_mode,
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
        io_priority: None,
        io_priority_key: None,
        efficiency_mode: None,
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
    .invoke_handler(tauri::generate_handler![get_process_status, optimize_processes, set_minimize_to_tray, send_notification, get_system_info])
    .manage(TrayState { minimize_to_tray: Mutex::new(false) })
    // 注册自动启动插件
    .plugin(tauri_plugin_autostart::init(
      tauri_plugin_autostart::MacosLauncher::LaunchAgent,
      Some(vec!["--hidden"]),
    ))
    .plugin(tauri_plugin_opener::init())
    .plugin(tauri_plugin_updater::Builder::new().build())
    .plugin(tauri_plugin_fs::init())
    .plugin(tauri_plugin_notification::init())
    .setup(|app| {
      let is_autostart = std::env::args().any(|a| a == "--hidden");

      if !is_autostart {
        if let Some(window) = app.get_webview_window("main") {
          let _ = window.show();
        }
      }

      let show_item = MenuItemBuilder::with_id("show", "显示主窗口").build(app)?;
      let settings_item = MenuItemBuilder::with_id("settings", "设置").build(app)?;
      let quit_item = MenuItemBuilder::with_id("quit", "退出").build(app)?;
      let menu = MenuBuilder::new(app)
        .item(&show_item)
        .item(&settings_item)
        .separator()
        .item(&quit_item)
        .build()?;

      let tray_icon = app.default_window_icon().cloned();

      let mut tray_builder = TrayIconBuilder::new()
        .tooltip("ACE 小助手")
        .menu(&menu);

      if let Some(icon) = tray_icon {
        tray_builder = tray_builder.icon(icon);
      }

      let _tray = tray_builder
        .on_menu_event(|app_handle, event| {
          match event.id().as_ref() {
            "show" => {
              if let Some(window) = app_handle.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
                let _ = app_handle.emit("tray-show-main", ());
              }
            }
            "settings" => {
              if let Some(window) = app_handle.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
                let _ = app_handle.emit("tray-open-settings", ());
              }
            }
            "quit" => {
              app_handle.exit(0);
            }
            _ => {}
          }
        })
        .on_tray_icon_event(|tray, event| {
          if let TrayIconEvent::DoubleClick { .. } = event {
            if let Some(window) = tray.app_handle().get_webview_window("main") {
              let _ = window.show();
              let _ = window.set_focus();
              let _ = tray.app_handle().emit("tray-show-main", ());
            }
          }
        })
        .build(app)?;

      if let Some(window) = app.get_webview_window("main") {
        let w = window.clone();
        window.on_window_event(move |event| {
          if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            let state = w.state::<TrayState>();
            let minimize = *state.minimize_to_tray.lock().unwrap();
            if minimize {
              let _ = w.hide();
              api.prevent_close();
            }
          }
        });
      }

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
