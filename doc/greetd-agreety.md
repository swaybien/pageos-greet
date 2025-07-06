# greetd-agreety 技术实现解析

- PJ568：

  > 请详细地解释 greetd-agreety 的工作原理和实现。

- qwen3：

  > greetd-agreety 是一个基于文本的 greeter（登录界面），它实现了 greetd 的 IPC 协议并与之交互以完成用户认证。以下是其工作原理和实现细节的详细解释：
  >
  > 1. **整体架构**
  >
  > `agreety` 是一个简单的命令行工具，主要负责：
  >
  > - 提示用户输入用户名和密码。
  > - 与 `greetd` 守护进程通信，通过 Unix 套接字发送认证请求。
  > - 根据 `greetd` 返回的响应处理认证流程。
  >
  > 2. **依赖库**
  >
  > 在 [Cargo.toml](/项目/greetd/agreety/Cargo.toml) 中定义了 `agreety` 的依赖项：
  >
  > ```toml
  > [dependencies]
  > greetd_ipc = { path = "../greetd_ipc", features = ["sync-codec"]}  # 提供与 greetd 通信的 IPC 协议
  > inish = { path = "../inish"}  # 用于解析配置文件
  > rpassword = "5.0"  # 安全地读取密码输入
  > getopts = "0.2"  # 解析命令行参数
  > enquote = "1.1"  # 处理带引号的字符串
  > nix = "0.27"  # 系统调用接口，例如 uname
  > ```
  >
  > 3. **核心功能**
  >
  > 3.1 **初始化与命令行参数解析**
  >
  > `main()` 函数首先解析命令行参数：
  >
  > ```rust
  > fn main() {
  >     let args: Vec<String> = env::args().collect();
  >     let program = args[0].clone();
  >     let mut opts = Options::new();
  >     opts.optflag("h", "help", "print this help menu");
  >     opts.optopt("c", "cmd", "command to run", "COMMAND");
  >     opts.optopt(
  >         "f",
  >         "max-failures",
  >         "maximum number of accepted failures",
  >         "FAILURES",
  >     );
  >     opts.optopt("u", "user", "restrict login to a specific user", "USER");
  >     let matches = match opts.parse(&args[1..]) {
  >         Ok(m) => m,
  >         Err(f) => {
  >             println!("{f}");
  >             print_usage(&program, opts);
  >             std::process::exit(1);
  >         }
  >     };
  >     if matches.opt_present("h") {
  >         print_usage(&program, opts);
  >         std::process::exit(0);
  >     }
  >
  >     let mut cmd = matches.opt_default("cmd", "");
  >     let max_failures: usize = match matches.opt_get("max-failures") {
  >         Ok(v) => v.unwrap_or(5),
  >         Err(e) => {
  >             eprintln!("unable to parse max failures: {e}");
  >             std::process::exit(1)
  >         }
  >     };
  >     let user: Option<String> = match matches.opt_get("user") {
  >         Ok(s) => s,
  >         Err(_) => None,
  >     };
  >
  >     if let Ok(issue) = get_issue() {
  >         print!("{issue}");
  >     }
  >
  >     let uts = uname().unwrap();
  >     for _ in 0..max_failures {
  >         match login(uts.nodename().to_str().unwrap(), &mut cmd, user.as_deref()) {
  >             Ok(LoginResult::Success) => break,
  >             Ok(LoginResult::Failure) => eprintln!("Login incorrect\n"),
  >             Err(e) => {
  >                 eprintln!("error: {e}");
  >                 std::process::exit(1);
  >             }
  >         }
  >     }
  > }
  > ```
  >
  > - 使用 `getopts` 解析命令行参数，支持帮助 (`--help`)、指定命令 (`--cmd`)、最大失败次数 (`--max-failures`) 和限制登录用户 (`--user`)。
  > - 如果提供了 `--help`，则打印使用说明并退出。
  > - 如果指定了 `--cmd`，则设置默认启动的命令。
  > - 如果指定了 `--user`，则限制只能登录特定用户。
  > - 最后进入循环，尝试登录直到成功或达到最大失败次数。
  >
  >   3.2 **获取系统信息**
  >
  > ```rust
  > fn get_distro_name() -> Result<String, Box<dyn std::error::Error>> {
  >     let os_release = fs::read_to_string("/etc/os-release")?;
  >     let parsed = inish::parse(&os_release)?;
  >     let general = parsed.get("").ok_or("no general section")?;
  >     maybe_unquote(general.get("PRETTY_NAME").ok_or("no pretty name")?)
  > }
  >
  > fn get_issue() -> Result<String, Box<dyn std::error::Error>> {
  >     let vtnr: usize = env::var("XDG_VTNR")
  >         .unwrap_or_else(|_| "0".to_string())
  >         .parse()
  >         .expect("unable to parse VTNR");
  >     let uts = uname()?;
  >     Ok(fs::read_to_string("/etc/issue")?
  >         .replace(
  >             "\\S",
  >             &get_distro_name().unwrap_or_else(|_| "Linux".to_string()),
  >         )
  >         .replace("\\l", &format!("tty{vtnr}"))
  >         .replace("\\s", uts.sysname().to_str().unwrap())
  >         .replace("\\r", uts.release().to_str().unwrap())
  >         .replace("\\v", uts.version().to_str().unwrap())
  >         .replace("\\n", uts.nodename().to_str().unwrap())
  >         .replace("\\m", uts.machine().to_str().unwrap())
  >         .replace("\\\\", "\\"))
  > }
  > ```
  >
  > - `get_distro_name()` 从 `/etc/os-release` 文件中读取操作系统的名称。
  > - `get_issue()` 读取 `/etc/issue` 文件，并替换其中的占位符（如 `\S`, `\l`, `\s` 等）为实际的系统信息，生成登录提示信息。
  >
  >   3.3 **登录逻辑**
  >
  > ```rust
  > enum LoginResult {
  >     Success,
  >     Failure,
  > }
  >
  > fn login(
  >     node: &str,
  >     cmd: &mut Option<String>,
  >     user: Option<&str>,
  > ) -> Result<LoginResult, Box<dyn std::error::Error>> {
  >     let username = if let Some(u) = user {
  >         println!("{node} login: {u}");
  >         u.to_string()
  >     } else {
  >         loop {
  >             let username = prompt_stderr(&format!("{node} login: "))?;
  >             if let Some(u) = username.strip_prefix('!') {
  >                 *cmd = Some(u.to_string());
  >                 eprintln!("Login command changed to: {u}");
  >                 continue;
  >             }
  >             break username;
  >         }
  >     };
  >
  >     let mut stream = UnixStream::connect(env::var("GREETD_SOCK")?)?;
  >
  >     let mut next_request = Request::CreateSession { username };
  >     let mut starting = false;
  >     loop {
  >         next_request.write_to(&mut stream)?;
  >
  >         match Response::read_from(&mut stream)? {
  >             Response::AuthMessage {
  >                 auth_message,
  >                 auth_message_type,
  >             } => {
  >                 let response = match auth_message_type {
  >                     AuthMessageType::Visible => Some(prompt_stderr(&auth_message)?),
  >                     AuthMessageType::Secret => Some(prompt_password_stderr(&auth_message)?),
  >                     AuthMessageType::Info => {
  >                         eprintln!("info: {auth_message}");
  >                         None
  >                     }
  >                     AuthMessageType::Error => {
  >                         eprintln!("error: {auth_message}");
  >                         None
  >                     }
  >                 };
  >
  >                 next_request = Request::PostAuthMessageResponse { response };
  >             }
  >             Response::Success => {
  >                 if starting {
  >                     return Ok(LoginResult::Success);
  >                 } else {
  >                     starting = true;
  >                     let command = match cmd {
  >                         Some(cmd) => cmd.clone(),
  >                         None => prompt_stderr("Command: ")?,
  >                     };
  >                     next_request = Request::StartSession {
  >                         env: vec![],
  >                         cmd: vec![command.to_string()],
  >                     }
  >                 }
  >             }
  >             Response::Error {
  >                 error_type,
  >                 description,
  >             } => {
  >                 Request::CancelSession.write_to(&mut stream)?;
  >                 match error_type {
  >                     ErrorType::AuthError => return Ok(LoginResult::Failure),
  >                     ErrorType::Error => return Err(format!("login error: {description:?}").into()),
  >                 }
  >             }
  >         }
  >     }
  > }
  > ```
  >
  > - `login()` 函数负责与 `greetd` 进行交互，完成用户的登录过程。
  > - 首先，如果指定了用户，则直接使用该用户；否则提示用户输入用户名。
  > - 创建一个 Unix 套接字连接到 `greetd`（路径由环境变量 `GREETD_SOCK` 指定）。
  > - 发送 `CreateSession` 请求，开始登录流程。
  > - 根据 `greetd` 返回的响应，处理不同的情况：
  >
  >   - 如果收到 `AuthMessage`，根据消息类型提示用户输入信息（可见或隐藏）。
  >   - 如果收到 `Success`，表示登录成功，继续启动会话。
  >   - 如果收到 `Error`，取消当前会话并返回错误信息。
  >
  >     3.4 **辅助函数**
  >
  > ```rust
  > fn maybe_unquote(s: &str) -> Result<String, Box<dyn std::error::Error>> {
  >     Ok(match s.chars().next() {
  >         Some('"') | Some('\'') => unquote(s)?,
  >         _ => s.to_string(),
  >     })
  > }
  >
  > fn prompt_stderr(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
  >     let stdin = io::stdin();
  >     let mut stdin_iter = stdin.lock().lines();
  >     eprint!("{prompt}");
  >     Ok(stdin_iter.next().ok_or("no input")??)
  > }
  > ```
  >
  > - `maybe_unquote()` 用于处理带引号的字符串，去除引号并返回原始内容。
  > - `prompt_stderr()` 用于在标准错误输出提示信息，并从标准输入读取用户的输入。
  >
  > 4. **总结**
  >
  > `agreety` 是一个轻量级的文本登录器，通过与 `greetd` 守护进程通信来完成用户认证。它的主要特点是简单、易用，并且能够灵活地与其他组件集成。通过 `greetd_ipc` 库提供的 IPC 协议，`agreety` 能够高效地与 `greetd` 进行交互，确保安全可靠的认证流程。

- PJ568：

  > 画一个用户操作流程图。

- qwen3：

  > 以下是 `greetd-agreety` 的用户操作流程图，展示了用户在使用 `agreety` 登录时的完整交互流程：
  >
  > ```
  > +-----------------------------+
  > |       用户启动 agreety      |
  > +------------+----------------+
  >              |
  >              v
  > +------------------------------+
  > | 显示 /etc/issue 登录提示信息 |
  > +------------+-----------------+
  >              |
  >              v
  > +-----------------------------+
  > | 输入用户名（可选）          |
  > | - 如果指定了用户，则跳过此步|
  > | - 否则提示输入用户名        |
  > +------------+----------------+
  >              |
  >              v
  > +------------------------------+
  > | 连接到 greetd Unix 套接字    |
  > | (路径由 GREETD_SOCK 环境变量)|
  > +------------+-----------------+
  >              |
  >              v
  > +-----------------------------+
  > | 发送 CreateSession 请求     |
  > | 包含用户名                  |
  > +------------+----------------+
  >              |
  >              v
  > +-----------------------------+
  > | 等待 greetd 返回响应        |
  > +------------+----------------+
  >              |
  >     +--------v--------------+---------------------------+
  >     |                       |                           |
  >     v                       v                           v
  > +-----------------------+  +---------------------+  +------------------+
  > | AuthMessage           |  | Success             |  | Error            |
  > | - 可见/隐藏/信息/错误 |  | - 开始启动会话      |  | - 取消会话       |
  > | - 提示用户输入        |  | - 用户确认后执行命令|  | - 返回失败或退出 |
  > +-----------------------+  +---------------------+  +------------------+
  >     |                       |
  >     v                       v
  > +-----------------------+  +-----------------+
  > | 回复 PostAuthMessage  |  | StartSession    |
  > | Response 请求         |  | - 执行指定命令  |
  > +-----------------------+  +-----------------+
  >              |
  >              v
  > +------------------------------------+
  > | 回到上述“等待 greetd 返回响应”步骤 |
  > +------------------------------------+
  > ```
  >
  > **流程说明**
  >
  > 1. **启动 `agreety`**  
  >    用户通过终端运行 `agreety` 命令，程序开始执行。
  >
  > 2. **显示登录提示信息**  
  >    `agreety` 读取 `/etc/issue` 文件并替换其中的占位符（如 `\S`, `\l`, `\s` 等），生成系统信息提示并输出到终端。
  >
  > 3. **输入用户名**
  >
  >    - 如果用户已在命令行中指定了用户名 (`--user`)，则直接使用该用户名。
  >    - 否则，程序提示用户输入用户名，并支持通过前缀 `!` 修改后续要执行的命令。
  >
  > 4. **连接到 `greetd`**  
  >    `agreety` 尝试通过 Unix 套接字连接到 `greetd` 守护进程（默认路径由环境变量 `GREETD_SOCK` 指定）。
  >
  > 5. **发送 `CreateSession` 请求**  
  >    程序向 `greetd` 发送一个包含用户名的 `CreateSession` 请求，表示开始一个新的登录会话。
  >
  > 6. **等待 `greetd` 响应**  
  >    根据 `greetd` 的返回结果，处理以下三种情况：
  >
  >    - **`AuthMessage`**: 表示需要进一步的身份验证。根据消息类型（可见、隐藏、信息、错误），提示用户输入相关信息。
  >    - **`Success`**: 表示认证已通过，程序将请求启动会话（`StartSession`），并执行指定的命令。
  >    - **`Error`**: 表示发生错误，程序取消当前会话，并根据错误类型决定是重试还是退出。
  >
  > 7. **完成登录，进入用户会话**  
  >    当所有身份验证步骤完成后，程序执行用户指定的命令（如 `/bin/sh`），进入用户的实际会话。
  >
  > 这个流程清晰地展示了 `agreety` 如何与 `greetd` 协作，确保用户能够安全、可靠地完成登录过程。

## 许可

`greetd-agreety` 使用 GPL-3.0-only 许可。
