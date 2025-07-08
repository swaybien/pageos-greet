# 网页 Greeter 实现设计

## 1. 架构设计

```mermaid
graph TD
    A[Web浏览器] -->|HTTP/WebSocket| B[Web服务器]
    B -->|UNIX Socket| C[greetd服务]
    B --> D[内嵌HTML资源]
```

## 2. 状态机流程图

```mermaid
flowchart TD
    subgraph pageos-greet
        START@{ shape: circle, label: "开始" } --> SERV[启动网页服务器]
        SERV -->|启动成功| BROW[启动图形界面和浏览器]
        START --> INIT[和 greetd 建立连接]
        SESS[创建 greetd 会话]
        SESS -->|发送 CreateSession 到 greetd| V[认证处理]
        SESS -->|从 greetd 收到 Success| PREP[启动会话准备]
        SESS -->|从 greetd 收到 Error| ERR[错误处理]
        V -->|从 greetd 收到 Success| PREP
        V -->|从 greetd 收到 Error| ERR
        PREP -->|发送 StartSession 到 greetd| BOOT[启动会话]
        ERR -->|发送 CancelSession 到 greetd 后重试| SESS
        BOOT --> STOP@{ shape: dbl-circ, label: "结束" }
    end

    subgraph 浏览器网页
        BROW -->|提供| DOM(网页加载)
        DOM -->|加载| UN[页面要求输入用户名]
        UN -->|“用户名信息”，且服务端已与 greetd 建立连接| SESS
        V -->|“提示信息”| AM[页面要求根据提示信息操作]
        AM -->|“用户根据提示信息提供的数据”| V
        ERR --> SHOW[显示错误信息或信息]
    end
```

## 3. Web 服务器实现

### 核心组件

- HTTP 服务器 (axum/warp)
- WebSocket 连接管理
- greetd IPC 客户端
- 会话状态机

### 端口配置

- 默认监听: 12801
- 可配置绑定地址

## 4. TODO API 设计

服务端返回当前认证状态:

```json
{
  "type": "Visible|Secret|Info|Error",
  "message": "提示信息"
}
```

> 因为 greetd 具体要获取信息未知，所以将 greetd 提供的提示（message）显示给用户，
> 让用户根据提示输入数据（用户名、密码、字符密钥等）。

浏览器网页提交用户响应:

```json
{
  "response": "用户输入"
}
```

## 5. 前端实现方案

### 内嵌 HTML

```rust
const INDEX_HTML: &str = r#"
<!DOCTYPE html>
<html>
  <head>
    <title>PageOS Greeter</title>
    <script>
      // WebSocket 客户端代码
    </script>
  </head>
  <body>
    <!-- 登录表单 -->
  </body>
</html>
"#;
```

### 交互流程

1. 页面加载后连接 WebSocket
2. 显示当前状态信息
3. 根据状态类型显示相应输入控件
4. 提交用户输入到接口

## 6. 与现有代码的集成

### 修改点

1. 将 main.rs 拆分为:

   - server.rs (Web 服务器)
   - ipc.rs (greetd 通信)
   - state.rs (会话状态机)

2. 共享核心逻辑:

```rust
async fn handle_auth_message(
    msg: AuthMessage
) -> Result<Response, Error> {
    // 复用现有认证处理逻辑
}
```

## 7. 安全考虑

- 仅允许本地访问(127.0.0.1)
- CSRF 保护
- 输入验证
- 会话超时
