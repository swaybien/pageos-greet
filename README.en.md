# pageos-greet

> [简体中文](README.md) | English

A login interface server written in Rust. Includes a minimal web server and built-in login page.

This project is a subproject of [pageos](https://github.com/swaybien/pageos).

Core principles:

- Minimal implementation by leveraging existing software as much as possible.

## Usage

This project depends on [greetd](https://git.sr.ht/~kennylevinsen/greetd),
and may require additional components like a desktop environment (e.g. [cage](https://github.com/cage-kiosk/cage))
and browser (e.g. [Firefox](https://github.com/mozilla-firefox/firefox)) depending on your needs.

Available command line arguments (excluding help):

- `--port`: Listening port, defaults to 12801
- `--page`: Path to custom login HTML file (recommended to store in `/usr/local/lib/pageos-greet/`).
  If not specified, uses the built-in HTML from `src/server.rs` ([HTML](doc/login.html))
- `--launch-command`: Command to run when starting pageos-greet, typically used to launch a kiosk browser to display the login page.
  This command will be terminated when pageos-greet exits.
- `--session-command`: Command to execute after successful login, defaults to `$SHELL` or `/usr/bin/bash`

Example:

```shell
pageos-greet \
  --port 12801 \
  --page /path/to/login.html \
  --launch-command "cage -s -- firefox --kiosk --no-remote http://127.0.0.1:12801" \
  --session-command "pageos-core -p 12800 --command \"cage -s -- firefox --kiosk --no-remote http://127.0.0.1:12800\""
```

When configuring `/etc/greetd/greetd.toml`, add commands to set up a temporary Home directory to prevent Firefox startup issues:

```toml
[default_session]
command = "bash -c 'mkdir -p /tmp/pageos-greet; export HOME=/tmp/pageos-greet; pageos-greet --launch-command \"cage -s -- firefox --kiosk --no-remote http://127.0.0.1:12801\" --session-command \"pageos-core -p 12800 --command \\"cage -s -- firefox --kiosk --no-remote http://127.0.0.1:12800\\"\"'"
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for contribution guidelines.```markdown

### Adaptation

_Ey bro, how do I adapt my page or Scratch to use the pageos-greet login manager?_

Yes! Bro! You just need to do these two things:

1. **Mode Detection**: Determine whether currently in login state.
   Check the domain and port at the very beginning of page initialization. If matched, initialize and enter the login page.
   For example, if your OS specifies the login page at `127.0.0.1:12801`, the condition would be:

   ```javascript
   if (
     (window.location.hostname === "127.0.0.1" ||
       window.location.hostname === "localhost") &&
     window.location.port === "12801"
   ) {
     // Initialize login page
   }
   ```

2. **Interface Adaptation**: Ensure your page/project contains all display (output) and input controls, and adapts to the login logic/message format.
   Reference:

   - [Example web page](doc/login.html)
   - [Adapter details](doc/design-adapter.md)
   - [Turbowarp 扩展](src/pageos-greet-extension.js)
   - [Scrach 适配细则](doc/Scrach%20适配细则.txt)

```

## Project Info

- License: MPL-2.0
- Repository: https://github.com/swaybien/pageos-greet
- Keywords: greetd, login, web-server, authentication
```
