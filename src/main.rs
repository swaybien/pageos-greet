// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod ipc;
mod server;

use crate::server::run_server;
use clap::Parser;
use std::process;
use tokio::process::Command;
use tracing::{error, info};

/// 命令行参数
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// 服务器监听端口
    #[arg(short, long, default_value_t = 12801)]
    port: u16,

    /// 用户会话启动命令
    #[arg(short, long)]
    session_command: Option<String>,

    /// 启动子进程命令 (父进程退出时会自动终止子进程)
    #[arg(short, long)]
    launch_command: Option<String>,
}

#[tokio::main]
async fn main() {
    // 初始化日志
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    info!("启动参数: {:?}", args);

    // 设置会话命令环境变量
    if let Some(cmd) = &args.session_command {
        info!("设置会话命令: {}", cmd);
        unsafe {
            std::env::set_var("SESSION_COMMAND", cmd);
        }
    } else {
        info!("使用默认会话命令: $SHELL");
    }

    // 启动子进程
    let child = if let Some(cmd) = &args.launch_command {
        info!("启动子进程: {}", cmd);
        Some(
            Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .kill_on_drop(true)
                .spawn()
                .expect("启动子进程失败")
        )
    } else {
        None
    };

    // 启动服务器
    info!("启动服务器，监听端口: {}", args.port);
    if let Err(e) = run_server(args.port).await {
        error!("服务器启动失败: {}", e);
        process::exit(1);
    }

    // 等待子进程退出
    if let Some(mut child) = child {
        match child.wait().await {
            Ok(status) => {
                info!("子进程已退出，状态码: {}", status);
                process::exit(0);
            }
            Err(e) => {
                error!("等待子进程时出错: {}", e);
                process::exit(1);
            }
        }
    }
}
