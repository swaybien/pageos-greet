# 代办

（注：有些文件的父目录在符号链接中，所以在列表中不可见，请正常访问读取）

1. **一次性直接读取以下文件：**
   `参考/greetd/man/greetd-ipc-7.scd`、`参考/greetd/greetd_ipc/src/lib.rs`、
   `参考/greetd/greetd_ipc/src/codec/tokio.rs`、`doc/design.md`、`Cargo.toml`、`src/main.rs`；
2. 和我讨论如何实现网页服务器和内嵌的 Web 页面，目前我的一些想法：

   - 将原先的输入（如收集用户输入的用户名、验证信息等）输出（如打印“用户名：”、“验证信息：”、“启动命令：”）做成 websocket 接口：
     - 服务端返回当前会话状态的：类型（Visible, Secret, Info, Error）、提示信息（一个字符串，如“用户名：”）；
     - 浏览器网页发送的数据为：用户输入（一个字符串，如用户名内容）；
   - 在 12801 端口启动网页服务器，浏览器访问 `http://localhost:12801` 可查看登录页面；
     - 12801 为默认端口，可通过 `--port` 参数指定；
   - 登录页面内嵌在 `src/main.rs` 中：
     - `DOMContentLoaded` 后初始化表单监听等……
   - 登录成功后启动用户会话，默认为环境变量 `$SHELL` 的值（如果不存在，则默认为 `/bin/sh`）；
     - 启动命令可通过 `--session-command` 参数指定；
3. 继续完成任务。
