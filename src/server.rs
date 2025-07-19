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

// 简单的 HTML 转义
fn html_escape(s: &str) -> String {
    s.replace('&', "&")
        .replace('<', "<")
        .replace('>', ">")
        .replace('\"', "\"")
        .replace('\'', "\'")
}

/// 启动 HTTP 服务器
pub async fn run_server(port: u16, html_content: String) -> Result<(), IpcError> {
    let app = Router::new()
        .route(
            "/",
            get(move || {
                let session_command = std::env::var("SESSION_COMMAND").unwrap_or_default();
                let escaped_session_command = html_escape(&session_command);
                async move {
                    let mut content = if html_content != "none".to_string() {
                        html_content.clone()
                    } else {
                        index_handler().await
                    };
                    // 确保替换所有占位符
                    // 优先替换带引号的占位符，再替换不带引号的
                    if escaped_session_command.is_empty() {
                        content = content
                            .replace("\"%SESSION_COMMAND%\"", "\"\"")
                            .replace("%SESSION_COMMAND%", "");
                    } else {
                        content = content
                            .replace(
                                "\"%SESSION_COMMAND%\"",
                                &format!("\"{}\"", &escaped_session_command),
                            )
                            .replace("%SESSION_COMMAND%", &escaped_session_command);
                    }
                    Html(content)
                }
            }),
        )
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
async fn index_handler() -> String {
    INDEX_HTML.to_string()
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
<html lang="zh-Hans-CN">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>PageOS 登录管理器</title>
    <link
      rel="icon"
      type="image/svg+xml"
      href="data:image/svg+xml;charset=utf-8;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSI1MCIgaGVpZ2h0PSI1MCI+PGNpcmNsZSBjeD0iMjUiIGN5PSIyNSIgcj0iMjAiIGZpbGw9ImdyZXkiIC8+PC9zdmc+"
    />
    <style>
      :root,
      body {
        width: 100%;
        height: 100%;
        background-color: gray;
        display: flex;
        align-items: center;
        justify-content: center;
      }
      h1 {
        font-size: xx-large;
      }
      summary {
        text-decoration: underline;
      }
      button,
      summary {
        cursor: pointer;
      }
      .auth-container {
        margin: 1rem;
        color: black;
        background-color: white;
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
        border: 0.1rem solid black;
        padding: 1rem;
      }
      .auth-container .auth-interface {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
      }
      .loading .auth-interface {
        display: none;
      }
      .auth-container input {
        box-sizing: border-box;
        width: 100%;
      }
      .auth-container .load {
        display: none;
        color: white;
        text-align: center;
        padding: 0.2rem;
        background-color: grey;
      }
      .loading .load {
        display: block;
      }
      .auth-container .log {
        border: 0.1rem solid black;
        padding: 0.5rem;
        font-size: smaller;
        max-height: 10rem;
        overflow-y: auto;
      }
      .auth-container .warn {
        color: orange;
      }
      .auth-container .error {
        color: red;
      }
    </style>
  </head>
  <body>
    <form id="auth" class="loading">
      <div class="auth-container">
        <h1>PageOS 登录</h1>
        <span class="load">正加载</span>
        <div class="auth-interface">
          <label class="prompt">
            <div>用户名：</div>
          </label>
          <input
            id="input"
            type="text"
            placeholder="请输入用户名"
            autocomplete="username"
          />
          <button type="submit" class="submit-btn">提交</button>
        </div>
        <details>
          <summary>高级</summary>
          <p>环境变量：</p>
          <input
            id="session-env"
            type="text"
            placeholder="LANG=zh_CN.UTF-8"
            autocomplete="session-env"
          />
          <p>启动命令：</p>
          <input
            id="session-cmd"
            type="text"
            value="%SESSION_COMMAND%"
            placeholder="pageos-core --command 'cage -s -- firefox --kiosk --no-remote http://127.0.0.1:12800'"
            autocomplete="session-cmd"
          />
          <p>日志：</p>
          <div class="log"></div>
        </details>
      </div>
    </form>

    <script>
      (function () {
        // 获取表单元素
        const auth = document.getElementById("auth");
        const loadElm = auth.querySelector(".auth-container span.load");
        const ai = auth.querySelector(".auth-container .auth-interface");
        const promptElm = auth.querySelector(".auth-container label.prompt");
        const inputElm = auth.querySelector(".auth-container input#input");
        const submitBtn = auth.querySelector(".auth-container .submit-btn");
        const statusLog = auth.querySelector(".auth-container .log");

        // 显示加载状态
        function showLoading(message = "正加载") {
          loadElm.textContent = message;
          auth.classList.add("loading");
        }

        // 隐藏加载状态
        function hideLoading() {
          auth.classList.remove("loading");
          inputElm.focus();
        }

        // 接口和状态
        const ws = new WebSocket(`ws://${window.location.host}/ws`);
        var currentState = "username"; // username: 未创建 greetd 会话, auth: 在会话中验证
        hideLoading();

        // 显示状态信息
        function logMessage(msg, type = "info") {
          const time = new Date().toLocaleTimeString();
          const div = document.createElement("div");
          div.textContent = `${time}：${msg}`;
          if (type === "error") div.className = "error";
          statusLog.appendChild(div);
          statusLog.scrollTop = statusLog.scrollHeight;
        }

        // 新增一条提示信息
        function addPrompt(text, type = "info") {
          const message = document.createElement("div");
          message.textContent = text;
          message.className = type;
          promptElm.appendChild(message);
          promptElm.scrollTop = promptElm.scrollHeight;
        }

        // 清空提示信息
        function clearPrompt() {
          promptElm.innerHTML = "";
        }

        // 设定页面文字
        function inerface_text(
          prompt = "用户名：",
          type = "info",
          input_type = "text",
          placeholder = "请输入用户名"
        ) {
          addPrompt(prompt, type);
          inputElm.type = input_type;
          inputElm.placeholder = placeholder;
          inputElm.value = "";
          hideLoading();
        }

        // 重置表单
        function resetForm() {
          clearPrompt();
          inerface_text();
          hideLoading();
          currentState = "username";
        }

        // 连接
        ws.onopen = () => {
          logMessage("已连接到服务器");
        };

        // 断开链接
        ws.onclose = () => {
          logMessage("连接已关闭", "warn");
          resetForm();
        };

        // 连接出错
        ws.onerror = (err) => {
          logMessage(`连接错误：${err}`, "error");
        };

        // 接收消息
        ws.onmessage = (event) => {
          try {
            const msg = JSON.parse(event.data);
            logMessage(`收到：${JSON.stringify(msg)}`);

            switch (msg.type) {
              case "AUTH_PROMPT":
                currentState = "auth";
                if (msg.message_type === "SECRET") {
                  inerface_text(msg.message, "info", "password", "请输入密钥");
                } else if (msg.message_type === "VISIBLE") {
                  currentState = "auth";
                  inerface_text(msg.message, "info", "text", "请输入文本");
                } else if (msg.message_type === "INFO") {
                  currentState = "auth";
                  addPrompt(msg.message);
                } else if (msg.message_type === "ERROR") {
                  logMessage(`✖ greetd 错误：${msg.message}`, "error");
                  addPrompt(msg.message, "error");
                }
                break;
              case "AUTH_SUCCESS":
                logMessage("✔ 登录成功! 正在启动会话……");
                showLoading("登录成功，正启动会话");
                const envStr = document.getElementById("session-env").value;
                const cmdStr = document.getElementById("session-cmd").value;
                // 将命令字符串分割为参数数组，处理带引号的参数
                const cmd =
                  cmdStr
                    .match(/[^\s"']+|"([^"]*)"|'([^']*)'/g)
                    ?.map((arg) => arg.replace(/^["']|["']$/g, "")) || [];
                // 将环境变量字符串分割为键值对数组
                const env =
                  envStr
                    .match(/[^\s"']+|"([^"]*)"|'([^']*)'/g)
                    ?.map((arg) => arg.replace(/^["']|["']$/g, "")) || [];
                ws.send(JSON.stringify({ type: "START_SESSION", cmd, env }));
                break;
              case "AUTH_ERROR":
                logMessage(`✖ 错误：${msg.reason}`, "error");
                resetForm();
                break;
            }
            hideLoading();
          } catch (e) {
            logMessage(`消息解析错误：${e}`, "error");
            addPrompt(`消息解析错误：${e}`, "error");
          }
        };

        document.getElementById("auth").addEventListener("submit", (e) => {
          e.preventDefault();
          console.log("表单提交已触发");
          if (currentState === "username") {
            const username = inputElm.value.trim();
            if (!username) {
              logMessage("请输入用户名", "warn");
              addPrompt("请输入用户名", "warn");
              return;
            }
            showLoading();
            ws.send(JSON.stringify({ type: "AUTH_REQUEST", username }));
          } else {
            const response = inputElm.value;
            if (!response) {
              logMessage("请输入信息", "warn");
              addPrompt("请输入信息", "warn");
              return;
            }
            showLoading();
            ws.send(JSON.stringify({ type: "AUTH_RESPONSE", response }));
          }
        });
      })();
    </script>
  </body>
</html>"#;
