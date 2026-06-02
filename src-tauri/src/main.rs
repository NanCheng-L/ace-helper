#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
  #[cfg(target_os = "windows")]
  {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    extern "system" {
      fn SetCurrentProcessExplicitAppUserModelID(app_id: *const u16) -> i32;
      fn IsUserAnAdmin() -> i32;
      fn ShellExecuteW(
        hwnd: *mut std::ffi::c_void,
        lpOperation: *const u16,
        lpFile: *const u16,
        lpParameters: *const u16,
        lpDirectory: *const u16,
        nShowCmd: i32,
      ) -> isize;
      fn CreateMutexW(
        lpMutexAttributes: *mut std::ffi::c_void,
        bInitialOwner: i32,
        lpName: *const u16,
      ) -> *mut std::ffi::c_void;
      fn GetLastError() -> u32;
      fn MessageBoxW(
        hWnd: *mut std::ffi::c_void,
        lpText: *const u16,
        lpCaption: *const u16,
        uType: u32,
      ) -> i32;
    }

    const ERROR_ALREADY_EXISTS: u32 = 183;

    let id: Vec<u16> = OsStr::new("com.ace-helper.app")
      .encode_wide()
      .chain(std::iter::once(0))
      .collect();
    unsafe { SetCurrentProcessExplicitAppUserModelID(id.as_ptr()) };

    // Release 模式下自动请求管理员权限
    if !cfg!(debug_assertions) && unsafe { IsUserAnAdmin() == 0 } {
      // 全局互斥锁：防止多个进程同时尝试提权（竞态条件）
      let mutex_name: Vec<u16> = OsStr::new("Global\\ace-helper-elevation-lock")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
      let _mutex = unsafe {
        CreateMutexW(std::ptr::null_mut(), 1, mutex_name.as_ptr())
      };
      let already_elevating = unsafe { GetLastError() } == ERROR_ALREADY_EXISTS;

      if already_elevating {
        // 另一个进程正在提权，当前进程直接退出，避免多实例
        std::process::exit(0);
      }

      let exe = std::env::current_exe().unwrap();
      let exe_str: Vec<u16> = OsStr::new(exe.to_str().unwrap())
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
      let op: Vec<u16> = OsStr::new("runas")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

      // 传递 --hidden 和 --elevated 参数
      let is_hidden = std::env::args().any(|a| a == "--hidden");
      let params = if is_hidden {
        "--hidden --elevated"
      } else {
        "--elevated"
      };
      let args: Vec<u16> = OsStr::new(params)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

      let result = unsafe {
        ShellExecuteW(
          std::ptr::null_mut(),
          op.as_ptr(),
          exe_str.as_ptr(),
          args.as_ptr(),
          std::ptr::null(),
          1,
        )
      };

      // UAC 被拒绝或安全策略拦截，静默退出
      std::process::exit(0);
    }
  }

  ace_helper_lib::run();
}
