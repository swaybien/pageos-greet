# 提示词

## 代办

（注：有些文件的父目录在符号链接中，所以在列表中不可见，请正常访问读取）

1. **一次性直接读取以下文件：**
   `参考/greetd/man/greetd-ipc-7.scd`、`参考/greetd/greetd_ipc/src/lib.rs`、`参考/greetd/greetd_ipc/src/codec/tokio.rs`、
   `参考/main.old.rs`、`doc/design.md`、`Cargo.toml`、`src/server.rs`、`src/ipc.rs`、`src/main.rs`；
2. 和我讨论如何实现网页服务器和内嵌的 Web 页面，目前我的一些想法：

   - 将 greetd-ipc 转接为 websocket 接口；
     - 原软件逻辑在 HTML 中实现（也就是说 ；
   - 在 12801 端口启动网页服务器，浏览器访问 `http://localhost:12801` 可查看登录页面；
     - 12801 为默认端口，可通过 `--port` 参数指定；
   - 登录页面内嵌在 Rust 代码中：
     - `DOMContentLoaded` 后初始化 ws 连接……
     - 当前阶段先暂时用 HTML 模拟命令行功能：
       - 打印命令行输出、用户输入；
     - 登录成功后启动用户会话，默认为环境变量 `$SHELL` 的值（如果不存在，则默认为 `/bin/sh`）；
       - 启动命令可通过 `--session-command` 参数指定；

3. 在 `src/ipc.rs` 实现将 greetd-ipc 转为 websocket。
4. 在 `src/server.rs` 实现网页服务器。
5. 在 `src/main.rs` 实现调用。

## 验证

（注：有些文件的父目录在符号链接中，所以在列表中不可见，请正常访问）

1. **一次性直接读取以下文件：**
   `参考/greetd/man/greetd-ipc-7.scd`、`参考/greetd/greetd_ipc/src/lib.rs`、`参考/greetd/greetd_ipc/src/codec/tokio.rs`、
   `参考/main.old.rs`、`doc/design.md`、`Cargo.toml`、`src/server.rs`、`src/ipc.rs`、`src/main.rs`；
2. 你觉得目前 pageos-greet 作为一个独立的 greeter 能正确地和 greetd 交互吗？
