use winapi::{
    shared::{minwindef::*, windef::*},
    um::winuser::*,
};
use std::process::Command;

#[derive(Debug)]
struct EnumParam {
    pid: u32,
    windows: Vec<HWND>,
}

extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let mut wpid = 0u32;
    let enum_param = unsafe {
        (lparam as *mut EnumParam).as_mut().unwrap()
    };
    unsafe {
        GetWindowThreadProcessId(hwnd, &mut wpid as *mut u32);
        if enum_param.pid == wpid {
            enum_param.windows.push(hwnd);
        }
    }
    1
}

fn get_windows_name(hwnd: HWND) -> String {
    unsafe {
        let mut result = Vec::with_capacity(256);
        let len = GetWindowTextW(hwnd, result.as_mut_ptr(), result.capacity() as _);
        result.set_len(len as usize);
        String::from_utf16(&result).unwrap()
    }
}

fn list_windows_by_process(pid: u32) -> Vec<HWND> {
    unsafe {
        let mut enum_param = EnumParam {
            pid,
            windows: Vec::with_capacity(16),
        };
        EnumWindows(Some(enum_proc), &mut enum_param as *mut _ as LPARAM);
        enum_param.windows
    }
}

fn test_window_open_time(exec_file: &str) {
    let mut c = Command::new(exec_file);
    let mut c = c.spawn().unwrap();
    let pid = c.id();
    let start_time = std::time::Instant::now();
    loop {
        std::thread::sleep(std::time::Duration::from_millis(0));
        let wnds = (list_windows_by_process(pid));
        let names = (wnds.iter().map(|&a| get_windows_name(a)).collect::<Vec<_>>());
        if wnds.len() > 0 && names.contains(&"Plain Craft Launcher 2\u{3000}".into()) {
            break;
        }
    }
    c.kill().unwrap();
    dbg!(start_time.elapsed().as_millis());
}

fn main() {
    // test_window_open_time("D:/Documents/Programs/Minecraft/SharpCraftLauncher-20210704-17dbd0-i686.exe");
    test_window_open_time("D:/Documents/Programs/Minecraft/Plain Craft Launcher 2.exe");
}
