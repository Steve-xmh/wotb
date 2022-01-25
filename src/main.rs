use mlua::prelude::*;
use serde::{Deserialize, Serialize};
use std::process::Command;
use winapi::{
    shared::{minwindef::*, windef::*},
    um::winuser::*,
};

#[derive(Debug)]
struct EnumParam {
    pid: u32,
    windows: Vec<HWND>,
}

#[derive(Debug, Deserialize)]
struct InputData {
    exec_file: String,
    exec_args: Vec<String>,
    bench_amount: usize,
    window_script: String,
}

#[derive(Debug, Serialize)]
struct OutputData {
    start_times: Vec<u128>,
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

fn get_window_long(hwnd: HWND, n_index: i32) -> i32 {
    unsafe {
        GetWindowLongW(hwnd, n_index)
    }
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

fn get_window_size(hwnd: HWND) -> (u32, u32, i32, i32) {
    unsafe {
        let mut rect = std::mem::zeroed();
        GetWindowRect(hwnd, &mut rect);
        (
            (rect.right - rect.left) as u32,
            (rect.bottom - rect.top) as u32,
            rect.right,
            rect.top
        )
    }
}

fn test_window_open_time_once(input: &InputData, lua: &Lua, window_script: &LuaFunction) -> u128 {
    let mut c = Command::new(&input.exec_file)
        .args(&input.exec_args)
        .spawn()
        .unwrap();
    let pid = c.id();
    let start_time = std::time::Instant::now();
    loop {
        std::thread::sleep(std::time::Duration::from_millis(0));
        let wnds = list_windows_by_process(pid);
        let wnds = wnds.iter().map(|&a| {
            let t = lua.create_table_with_capacity(0, 2).unwrap();
            let name = get_windows_name(a);
            let name = lua.create_string(&name).unwrap();
            let size_t = lua.create_table_with_capacity(0, 2).unwrap();
            let pos_t = lua.create_table_with_capacity(0, 2).unwrap();
            let size = get_window_size(a);
            size_t.set("width", size.0).unwrap();
            size_t.set("height", size.1).unwrap();
            pos_t.set("x", size.2).unwrap();
            pos_t.set("y", size.3).unwrap();
            t.set("name", name).unwrap();
            t.set("position", pos_t).unwrap();
            t.set("size", size_t).unwrap();
            t.set("style", get_window_long(a, GWL_STYLE)).unwrap();
            t.set("exstyle", get_window_long(a, GWL_EXSTYLE)).unwrap();
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

fn test_window_open_time(input: &InputData) -> OutputData {
    let mut v = Vec::with_capacity(20);
    let lua = Lua::new();
    let window_script = lua.load(&input.window_script).into_function().unwrap();
    for _ in 0..input.bench_amount {
        v.push(test_window_open_time_once(input, &lua, &window_script));
    }
    {
        let mut v = v.to_owned();
        v.sort_unstable();
    
        dbg!(v.iter().sum::<u128>() / 20);
        dbg!(v.first());
        dbg!(v.last());
    }
    OutputData { start_times: v }
}

fn main() {
    let mut args = std::env::args().collect::<Vec<_>>();
    if args.len() < 3 {
        if let Some(exec) = args.first() {
            println!(
                "{} [INPUT] [OUTPUT]",
                exec
            );
        } else {
            println!(
                "{} [INPUT] [OUTPUT]",
                std::env::current_exe()
                    .unwrap_or_else(|_| "wotb".into())
                    .display()
            );
        }
        return;
    }
    let output_path = args.pop().unwrap();
    let input_path = args.pop().unwrap();
    let input_data = serde_json::from_str(&std::fs::read_to_string(input_path).unwrap()).unwrap();
    let result = test_window_open_time(&input_data);
    std::fs::write(output_path, serde_json::to_string_pretty(&result).unwrap()).unwrap();
}
