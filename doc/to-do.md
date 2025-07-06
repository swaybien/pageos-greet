# 代办

可供参考的到项目克隆的符号链接（所以在文件列表可能中看不到，但可正常访问其文件）：

- [greetd-agreety](/参考/greetd/agreety)
- [gtkgreet](/参考/gtkgreet)
- [tuigreet](/参考/tuigreet)
- [ReGreet](/参考/ReGreet)

阶段：

1. 参考 [greetd-agreety 技术实现解析](/doc/greetd-agreety.md)实现一个基本的 greeter：

   1. 终端显示`用户名：`，获取用户名；
   2. 连接到 greetd Unix 套接字（路径由 GREETD_SOCK 环境变量控制）；
   3. 发送包含用户名的 CreateSession 请求创建会话；
   4. 等待 greetd 返回响应：

      - 如果 greetd 返回错误，则显示错误并返回第一步；
      - 如果 greetd 返回成功，则跳到第五步；
      - 如果 greetd 响应验证信息（可见/隐藏/信息/错误），根据指示提示用户输入；
        - 发送包含用户输入的回应信息，又回到了第四步。

   5. 如果不存在通过参数 `--session-command` 获取的用户会话启动命令，终端显示`启动命令：`，获取启动命令；
      如果存在，则使用该启动命令启动会话。

2. 实现网页服务器和简单内嵌的 Web 页面，获取用户名、验证信息，返回给 greeter；
3. 实现 greeter 执行 `--launch-command` 获取的界面显示命令；
4. 详细通读 [tuigreet](/参考/tuigreet) 的自述、[文档](/doc/tuigreet.md)、Cargo.toml 或其他描述项目依赖的文件和源代码，
   了解它的依赖引入方法、实现原理、与协议沟通的方法。
   实现多次启动共用实例。
5. 最终期望和需求：

   以下是除帮助参数外可用参数：

   ```sh
   pageos-greet \
     --port 12801 \
     --page /path/to/login.html \
     --launch-command "cage -s -- firefox --kiosk http://localhost:12801/login" \
     --session-command "pageos-core -p 12800 --command \"cage -s -- firefox --kiosk --no-remote http://localhost:12800\""
   ```

   - 默认的 HTML 登录页面内置在主程序 `pageos-greet` 中；
   - 如果需要自定义登录界面，则使用参数传入自定义 HTML；

步骤：

1. 详细通读 [greetd 的自述](/参考/greetd/README.md)、[greetd-ipc 源码文件夹](/参考/greetd/greetd_ipc/src/)下所有文件
   和 [greetd-ipc(7) 文档](/参考/greetd/man/greetd-ipc-7.scd)，了解 greetd 的工作原理和协议。
2. 和用户探讨下一步要实现的阶段的方案，讨论完后详细地写入 `/doc/design.md`。
3. 实现`阶段一`。
