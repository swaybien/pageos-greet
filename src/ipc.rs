// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use greetd_ipc::{Request, Response, codec::TokioCodec};
use std::env;
use thiserror::Error;
use tokio::net::UnixStream;

/// WebSocket 消息格式
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WsMessage {
    /// 从客户端到服务器
    AuthRequest {
        username: String,
    },
    AuthResponse {
        response: String,
    },
    StartSession {
        cmd: Vec<String>,
        env: Vec<String>,
    },

    /// 从服务器到客户端
    AuthMessage {
        message: String,
        message_type: String,
    },
    AuthSuccess,
    AuthError {
        reason: String,
    },
}

#[derive(Error, Debug)]
pub enum IpcError {
    #[error("IPC 连接错误: {0}")]
    Connection(#[from] std::io::Error),
    #[error("消息序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("协议错误: {0}")]
    Protocol(String),
    #[error("WebSocket 错误: {0}")]
    WebSocket(String),
    #[error("IPC 协议错误: {0}")]
    IpcProtocol(#[from] greetd_ipc::codec::Error),
}

/// 连接到 greetd IPC 服务
pub async fn connect_to_greetd() -> Result<UnixStream, IpcError> {
    let sock_path = env::var("GREETD_SOCK")
        .map_err(|_| IpcError::Protocol("GREETD_SOCK 环境变量未设置".into()))?;
    UnixStream::connect(&sock_path).await.map_err(Into::into)
}

/// WebSocket 消息 -> greetd IPC 请求
pub fn ws_to_ipc(msg: WsMessage) -> Result<Request, IpcError> {
    match msg {
        WsMessage::AuthRequest { username } => Ok(Request::CreateSession { username }),
        WsMessage::AuthResponse { response } => Ok(Request::PostAuthMessageResponse {
            response: Some(response),
        }),
        WsMessage::StartSession { cmd, env } => Ok(Request::StartSession { cmd, env }),
        _ => Err(IpcError::Protocol("无效的 WebSocket 消息类型".into())),
    }
}

/// greetd IPC 响应 -> WebSocket 消息
pub fn ipc_to_ws(resp: Response) -> WsMessage {
    match resp {
        Response::AuthMessage {
            auth_message_type,
            auth_message,
        } => WsMessage::AuthMessage {
            message_type: format!("{:?}", auth_message_type).to_uppercase(),
            message: auth_message,
        },
        Response::Success => WsMessage::AuthSuccess,
        Response::Error {
            error_type: _,
            description,
        } => WsMessage::AuthError {
            reason: description,
        },
    }
}

/// 处理 WebSocket 连接，转发 greetd IPC 消息
pub async fn handle_websocket(
    mut ws: axum::extract::ws::WebSocket,
    mut ipc_conn: UnixStream,
) -> Result<(), IpcError> {
    while let Some(msg) = ws.recv().await {
        let msg = msg.map_err(|e| IpcError::Protocol(e.to_string()))?;
        let text = msg
            .into_text()
            .map_err(|e| IpcError::Protocol(e.to_string()))?;
        let ws_msg: WsMessage = serde_json::from_str(&text)?;

        let ipc_msg = ws_to_ipc(ws_msg)?;
        ipc_msg
            .write_to(&mut ipc_conn)
            .await
            .map_err(IpcError::from)?;

        let resp = Response::read_from(&mut ipc_conn)
            .await
            .map_err(IpcError::from)?;
        let ws_msg = ipc_to_ws(resp);
        let json = serde_json::to_string(&ws_msg)?;

        // 如果是 StartSession 成功响应，则退出进程
        if let WsMessage::AuthSuccess = ws_msg {
            if let Request::StartSession { .. } = ipc_msg {
                std::process::exit(0);
            }
        }

        ws.send(axum::extract::ws::Message::Text(json.into()))
            .await
            .map_err(|e| IpcError::WebSocket(e.to_string()))?;
    }
    Ok(())
}
