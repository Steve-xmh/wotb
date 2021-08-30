use std::process::Command;
use winapi::{
    shared::{minwindef::*, windef::*},
    um::winuser::*,
};
use serde::{Deserialize, Serialize};
use mlua::prelude::*;

#[derive(Debug)]
struct EnumParam {
    pid: u32,
    windows: Vec<HWND>,
}

#[derive(Debug, Deserialize)]
struct InputData {
    exec_file: String,
    window_script: String,
}

#[derive(Debug, Serialize)]
struct OutputData {

}

extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let mut wpid = 0u32;
    let enum_param = unsafe { (lparam as *mut EnumParam).as_mut().unwrap() };
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

fn get_window_size(hwnd: HWND) -> (u32, u32) {
    unsafe {
        let mut rect = std::mem::zeroed();
        GetWindowRect(hwnd, &mut rect);
        ((rect.right - rect.left) as u32, (rect.bottom - rect.top) as u32)
    }
}

fn test_window_open_time_once(input: &InputData, lua: &Lua, window_script: &LuaFunction) -> u128 {
    let mut c = Command::new(&input.exec_file);
    let mut c = c.spawn().unwrap();
    let pid = c.id();
    let start_time = std::time::Instant::now();
    loop {
        std::thread::sleep(std::time::Duration::from_millis(0));
        let wnds = list_windows_by_process(pid);
        let wnds = wnds
            .iter()
            .map(|&a| {
                let t = lua.create_table_with_capacity(0, 2).unwrap();
                let name = get_windows_name(a);
                let name = lua.create_string(&name).unwrap();
                let size_t = lua.create_table_with_capacity(0, 2).unwrap();
                let size = get_window_size(a);
                size_t.set("width", size.0).unwrap();
                size_t.set("height", size.1).unwrap();
                t.set("name", name).unwrap();
                t.set("size", size_t).unwrap();
                t
            });
        let wnds_t = lua.create_sequence_from(wnds).unwrap();
        lua.globals().set("windows", wnds_t).unwrap();
        if window_script.call(()).unwrap_or(false) {
            break;
        }
    }
    c.kill().unwrap();
    dbg!(start_time.elapsed().as_millis())
}

fn test_window_open_time(input: &InputData) {
    let mut v = Vec::with_capacity(20);
    let mut lua = Lua::new();
    let window_script = lua.load(&input.window_script).into_function().unwrap();
    for _ in 0..20 {
        v.push(
            test_window_open_time_once(&input, &lua, &window_script)
        );
    }
    v.sort_unstable();
    dbg!(v.iter().fold(0, |acc, &x| acc + x) / 20 as u128);
    dbg!(v.first());
    dbg!(v.last());
}

fn main() {
    let mut args = std::env::args().collect::<Vec<_>>();
    let output_path = args.pop().unwrap();
    let input_path = args.pop().unwrap();
    let input_data = serde_json::from_str(&std::fs::read_to_string(input_path).unwrap()).unwrap();
    test_window_open_time(&input_data);
}
