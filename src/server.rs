// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::ipc::{IpcError, connect_to_greetd, handle_websocket};
use axum::{
    Router,
    extract::ws::{WebSocket, WebSocketUpgrade},
    response::Html,
    routing::get,
};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::{error, info};

/// 启动 HTTP 服务器
pub async fn run_server(port: u16) -> Result<(), IpcError> {
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/ws", get(websocket_handler));

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    info!("正在绑定TCP监听端口: {}", addr);
    let listener = TcpListener::bind(addr).await.map_err(|e| {
        error!("无法绑定端口 {}: {}", port, e);
        IpcError::Protocol(e.to_string())
    })?;
    info!("服务器启动成功，监听端口: {}", port);
    axum::serve(listener, app.into_make_service())
        .await
        .map_err(|e| {
            error!("服务器运行错误: {}", e);
            IpcError::Protocol(e.to_string())
        })?;
    Ok(())
}

/// 处理首页请求
async fn index_handler() -> Html<&'static str> {
    Html(INDEX_HTML)
}

/// 处理 WebSocket 升级请求
async fn websocket_handler(ws: WebSocketUpgrade) -> impl axum::response::IntoResponse {
    ws.on_upgrade(handle_socket)
}

/// 处理 WebSocket 连接
async fn handle_socket(socket: WebSocket) {
    // 连接到 greetd IPC
    match connect_to_greetd().await {
        Ok(ipc_conn) => {
            if let Err(e) = handle_websocket(socket, ipc_conn).await {
                error!("WebSocket 处理错误: {}", e);
            }
        }
        Err(e) => error!("无法连接到 greetd: {}", e),
    }
}
const INDEX_HTML: &str = r#"<!DOCTYPE html>
<html>
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>PageOS 登录管理器</title>
    <style>
      :root {
        --primary-color: #2563eb;
        --error-color: #dc2626;
      }
      body {
        font-family: "Segoe UI", system-ui, sans-serif;
        max-width: 600px;
        margin: 0 auto;
        padding: 2rem;
        background-color: #f8fafc;
      }
      #auth-container {
        border: 1px solid #e2e8f0;
        padding: 2rem;
        border-radius: 0.5rem;
        background: white;
        box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
      }
      h1 {
        color: var(--primary-color);
        margin-bottom: 1.5rem;
        text-align: center;
      }
      input {
        box-sizing: border-box;
        width: 100%;
        padding: 0.75rem;
        margin: 0.5rem 0 1rem;
        border: 1px solid #e2e8f0;
        border-radius: 0.375rem;
        font-size: 1rem;
      }
      button {
        width: 100%;
        padding: 0.75rem;
        background-color: var(--primary-color);
        color: white;
        border: none;
        border-radius: 0.375rem;
        font-size: 1rem;
        cursor: pointer;
        transition: background-color 0.2s;
      }
      button:hover {
        background-color: #1d4ed8;
      }
      #status-log {
        margin-top: 1.5rem;
        padding: 1rem;
        background: #f1f5f9;
        border-radius: 0.375rem;
        min-height: 100px;
        max-height: 300px;
        overflow-y: auto;
        font-family: monospace;
        font-size: 0.875rem;
      }
      .error {
        color: var(--error-color);
      }
      @media (max-width: 640px) {
        body {
          padding: 1rem;
        }
        #auth-container {
          padding: 1.5rem;
        }
      }
    </style>
  </head>
  <body>
    <div id="auth-container">
      <h1>PageOS 登录</h1>
      <div id="prompt">请输入用户名:</div>
      <input
        id="username"
        type="text"
        placeholder="用户名"
        autocomplete="username"
      />
      <input
        id="password"
        type="password"
        placeholder="密码"
        style="display: none"
        autocomplete="current-password"
      />
      <button id="submit-btn">提交</button>
      <div id="status-log"></div>
    </div>

    <script>
      const ws = new WebSocket(`ws://${window.location.host}/ws`);
      const statusLog = document.getElementById("status-log");
      const submitBtn = document.getElementById("submit-btn");
      let currentState = "username";

      ws.onopen = () => {
        logMessage("已连接到服务器", "info");
      };

      ws.onclose = () => {
        logMessage("连接已关闭", "warn");
      };

      ws.onerror = (err) => {
        logMessage(`连接错误: ${err}`, "error");
      };

      ws.onmessage = (event) => {
        try {
          const msg = JSON.parse(event.data);
          logMessage(`收到: ${JSON.stringify(msg)}`, "info");

          switch (msg.type) {
            case "AUTH_PROMPT":
              if (msg.message_type === "secret") {
                document.getElementById("password").style.display = "block";
                document.getElementById("username").style.display = "none";
                currentState = "password";
                submitBtn.textContent = "登录";
              }
              document.getElementById("prompt").textContent = msg.message;
              break;
            case "AUTH_SUCCESS":
              logMessage("✔ 登录成功! 正在启动会话...", "success");
              submitBtn.disabled = true;
              submitBtn.textContent = "登录成功";
              break;
            case "AUTH_ERROR":
              logMessage(`✖ 错误: ${msg.reason}`, "error");
              resetForm();
              break;
          }
        } catch (e) {
          logMessage(`消息解析错误: ${e}`, "error");
        }
      };

      submitBtn.addEventListener("click", () => {
        if (currentState === "username") {
          const username = document.getElementById("username").value.trim();
          if (!username) {
            logMessage("请输入用户名", "warn");
            return;
          }
          ws.send(JSON.stringify({ type: "AUTH_REQUEST", username }));
        } else {
          const password = document.getElementById("password").value;
          if (!password) {
            logMessage("请输入密码", "warn");
            return;
          }
          ws.send(
            JSON.stringify({ type: "AUTH_RESPONSE", response: password })
          );
        }
      });

      function logMessage(msg, type = "info") {
        const time = new Date().toLocaleTimeString();
        const div = document.createElement("div");
        div.textContent = `${time}: ${msg}`;
        if (type === "error") div.className = "error";
        statusLog.appendChild(div);
        statusLog.scrollTop = statusLog.scrollHeight;
      }

      function resetForm() {
        document.getElementById("username").style.display = "block";
        document.getElementById("password").style.display = "none";
        document.getElementById("username").value = "";
        document.getElementById("password").value = "";
        submitBtn.textContent = "提交";
        currentState = "username";
      }
    </script>
  </body>
</html>"#;
