// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use greetd_ipc::{AuthMessageType, ErrorType, Request, Response, codec::TokioCodec};
use rpassword::prompt_password;
use std::env;
use std::io::{self, Write};
use std::process;
use std::time::Duration;
use tokio::net::UnixStream;

const MAX_RETRIES: u8 = 3;
const RETRY_DELAY: u64 = 2;

struct GreeterState {
    retry_count: u8,
    username: String,
    stream: Option<UnixStream>,
}

impl GreeterState {
    fn new() -> Self {
        Self {
            retry_count: 0,
            username: String::new(),
            stream: None,
        }
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut state = GreeterState::new();
    state.username = prompt_input("用户名：")?;

    loop {
        match try_login_flow(&mut state).await {
            Ok(()) => break,
            Err(e) => {
                handle_error(&mut state, &e);

                if state.retry_count >= MAX_RETRIES {
                    eprintln!("超过最大重试次数");
                    process::exit(1);
                }

                // 根据错误类型决定重试策略
                if !should_retry_immediately(&e) {
                    let delay =
                        Duration::from_secs(RETRY_DELAY * 2u64.pow(state.retry_count as u32));
                    eprintln!("等待 {:?} 后重试...", delay);
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }
    Ok(())
}

async fn try_login_flow(state: &mut GreeterState) -> io::Result<()> {
    let stream = connect_to_greetd().await?;
    state.stream = Some(stream);
    state.retry_count = 0;

    let mut request = Request::CreateSession {
        username: state.username.clone(),
    };
    let mut starting_session = false;

    loop {
        let stream = state.stream.as_mut().unwrap();
        request
            .write_to(stream)
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        match Response::read_from(stream)
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
        {
            Response::AuthMessage {
                auth_message_type,
                auth_message,
            } => {
                let response = match auth_message_type {
                    AuthMessageType::Visible => Some(prompt_input(&auth_message)?),
                    AuthMessageType::Secret => {
                        let password = prompt_password(&auth_message)
                            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                        Some(password)
                    }
                    AuthMessageType::Info => {
                        eprintln!("信息: {}", auth_message);
                        None
                    }
                    AuthMessageType::Error => {
                        eprintln!("错误: {}", auth_message);
                        None
                    }
                };

                request = Request::PostAuthMessageResponse { response };
            }
            Response::Success => {
                if starting_session {
                    return Ok(());
                }

                starting_session = true;
                let cmd = env::var("SHELL")
                    .map(|s| vec![s])
                    .unwrap_or_else(|_| vec!["/bin/bash".to_string()]);

                // 安全的环境变量白名单
                let safe_env = vec!["LANG", "PATH", "DISPLAY", "XAUTHORITY", "WAYLAND_DISPLAY"];
                let env: Vec<String> = env::vars()
                    .filter(|(k, _)| safe_env.contains(&k.as_str()))
                    .map(|(k, v)| format!("{k}={v}"))
                    .collect();

                request = Request::StartSession { cmd, env };
            }
            Response::Error {
                error_type,
                description,
            } => {
                match error_type {
                    ErrorType::AuthError => {
                        // 认证错误时取消会话
                        Request::CancelSession
                            .write_to(stream)
                            .await
                            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                        return Err(io::Error::new(io::ErrorKind::PermissionDenied, description));
                    }
                    ErrorType::Error => {
                        Request::CancelSession
                            .write_to(stream)
                            .await
                            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                        return Err(io::Error::new(io::ErrorKind::Other, description));
                    }
                }
            }
        }
    }
}

async fn connect_to_greetd() -> io::Result<UnixStream> {
    let sock_path = env::var("GREETD_SOCK")
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "GREETD_SOCK环境变量未设置"))?;

    for retry in 0..MAX_RETRIES {
        match UnixStream::connect(&sock_path).await {
            Ok(stream) => return Ok(stream),
            Err(_e) if retry < MAX_RETRIES - 1 => {
                eprintln!("连接失败，重试中... ({}/{})", retry + 1, MAX_RETRIES);
                tokio::time::sleep(Duration::from_secs(RETRY_DELAY)).await;
            }
            Err(e) => {
                return Err(io::Error::new(
                    io::ErrorKind::ConnectionRefused,
                    format!("连接greetd服务失败: {} (sock={})", e, sock_path),
                ));
            }
        }
    }
    unreachable!()
}

fn prompt_input(prompt: &str) -> io::Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn handle_error(state: &mut GreeterState, error: &io::Error) {
    eprintln!("错误: {}", error);
    state.retry_count += 1;
    state.stream = None;
}

// 错误分类处理
fn should_retry_immediately(error: &io::Error) -> bool {
    match error.kind() {
        io::ErrorKind::PermissionDenied => true, // 认证错误
        io::ErrorKind::InvalidData => true,      // 无效凭据
        _ => false,                              // 连接或其他错误
    }
}
