# pageos-greet

Rust 语言编写的登录界面服务器。包含一个最小网页服务器和内置的登录页面。

核心思想：

- 尽量依赖现有软件，保持本项目为最小实现。

## 使用方法

以下是除帮助参数外可用参数：

```sh
pageos-greet \
  --port 12801 \
  --page /path/to/login.html \
  --launch-command "cage -s -- firefox --kiosk http://localhost:12801/login" \
  --session-command "pageos-core -p 12800 --command \"cage -s -- firefox --kiosk --no-remote http://localhost:12800\""
```

## 指标

- 登录界面：
  - 显示管理器采用 `greetd`；
  - 默认的 HTML 登录页面内置在主程序 `pageos-greet` 中；
  - 如果需要自定义登录界面，则使用参数传入自定义 HTML；
