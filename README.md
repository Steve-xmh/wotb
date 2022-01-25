# WOTB

Window Open Time Benchmark

---

测试软件启动时间的一个小工具，通过测试其有没有窗口（可过滤）来辨别是否启动成功。

## 如何使用

需要输入一个元数据 JSON，格式如下：

```jsonc
{
    "exec_file": "test.exe",               // 要测试的软件的文件路径
    "exec_args": [],                       // 任何你想传递给这个软件的参数，不需要则保留空数组
    "bench_amount": 20,                    // 需要测试的次数
    "window_script": "return #windows > 0" // 用于根据获取到的窗口信息判断是否已启动的 Lua 脚本，详情如下
}
```

然后执行时间测试：

```bash
wotb [输入的元数据路径] [导出JSON结果的路径]
```

## 判断窗口的 Lua 脚本

元数据中 `window_script` 字段是一个 Lua 脚本，会在工具每次获取到所有所启动进程的所有窗口时被调用。
如果这个脚本返回了 `true` 则代表所需窗口已创建显示，工具将判定此为启动成功并记录时间。

在执行时，脚本所属环境会有一个 `windows` 全局变量，是一个存储了所有窗口的属性的数组表。
每个窗口信息结构如下：

```lua
local window = {
    name = "",        -- 窗口的标题 （使用 GetWindowTextW 的返回值）
    style = 0,        -- GetWindowLongW(hwnd, GWL_STYLE) 的返回值
    exstyle = 0,      -- GetWindowLongW(hwnd, GWL_EXSTYLE) 的返回值
    position = {      -- 窗口的位置，单位为像素
        x = 400,      -- 窗口的水平位置
        y = 300,      -- 窗口的垂直位置
    }
    size = {          -- 窗口的大小，单位为像素
        width = 400,  -- 窗口的宽度
        height = 300, -- 窗口的高度
    }
}
```

如果只需要检测是否有窗口，那么直接写入 `return #windows > 0` 即可

## 导出的结果 JSON

```jsonc
{
    "start_times": [123, 234] // 每次测试的启动时长结果，以毫秒为单位
}
```
