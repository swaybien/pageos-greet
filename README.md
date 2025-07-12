# pageos-greet

> 简体中文 | [English](README.en.md)

Rust 语言编写的登录界面服务器。包含一个最小网页服务器和内置的登录页面。

本项目是 [pageos](https://github.com/swaybien/pageos) 项目的一个子项目。

核心思想：

- 尽量依赖现有软件，保持本项目为最小实现。

## 使用方法

本项目依赖 [greetd](https://git.sr.ht/~kennylevinsen/greetd)，
根据您的需求可能还需搭配其它桌面环境（如 [cage](https://github.com/cage-kiosk/cage)）和浏览器（如 [Firefox](https://github.com/mozilla-firefox/firefox)）。

以下是除帮助参数外可用参数：

- `--port`：监听端口，默认为 12801；
- `--page`：传入自定义登录界面 HTML 文件路径（建议存放在 `/usr/local/lib/pageos-greet/` 下），未指定时默认使用内嵌在 `src/server.rs` 中的 [HTML](doc/login.html)；
- `--launch-command`：启动 pageos-greet 时运行的命令，一般用来启动 kiosk 浏览器界面以显示登录页面，在 pageos-greet 退出一同关闭；
- `--session-command`：用户登录成功后执行的命令，默认为 `$SHELL` 的值或 `/usr/bin/bash`；

案例：

```shell
pageos-greet \
  --port 12801 \
  --page /path/to/login.html \
  --launch-command "cage -s -- firefox --kiosk --no-remote http://127.0.0.1:12801" \
  --session-command "pageos-core -p 12800 --command \"cage -s -- firefox --kiosk --no-remote http://127.0.0.1:12800\""
```

当配置 `/etc/greetd/greetd.toml` 时，添加了设置临时 Home 的命令以防止 firefox 无法正常启动。

```toml
[default_session]
command = "bash -c 'mkdir -p /tmp/pageos-greet; export HOME=/tmp/pageos-greet; pageos-greet --launch-command \"cage -s -- firefox --kiosk --no-remote http://127.0.0.1:12801\" --session-command \"pageos-core -p 12800 --command \\"cage -s -- firefox --kiosk --no-remote http://127.0.0.1:12800\\"\"'"
```

## 贡献

详见 [CONTRIBUTING.md](CONTRIBUTING.md)。

### 适配

_不是哥们，那我要怎么适配我的页面或 Scratch 以使用 pageos-greet 这个登录管理器？_

是！哥们！您只需要做到以下两点：

1. 模式判断：判断当前是否处于登录状态；

   在页面初始化最开始检查域名和端口号，如果匹配则初始化进入登录页面。
   假如您的操作系统规定登录页面在 127.0.0.1:12801，则判断条件为：

   ```javascript
   if (
     (window.location.hostname === "127.0.0.1" ||
       window.location.hostname === "localhost") &&
     window.location.port === "12801"
   ) {
     // 初始化登录页面
   }
   ```
2. 接口适配：保证您的页面或项目中包含所有的显示（输出）和输入控件并适配了登录逻辑和消息格式；
   可参考：
   
   - [示例网页](doc/login.html)
   - [适配详解](doc/design-adapter.md)
   - [Turbowarp 扩展](src/pageos-greet-extension.js)
   - [Scrach 适配细则](doc/Scrach%20适配细则.txt)

## 项目信息

- 许可：MPL-2.0
- 仓库：https://github.com/swaybien/pageos-greet
- 关键词：greetd, login, web-server, authentication
