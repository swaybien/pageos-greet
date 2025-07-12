// 本扩展需要非沙盒环境运行：websocket
// This extension requires disabling sandboxing: websocket
class PageOSLoginExtension {
  constructor() {
    this.ws = null;
    this.connected = false;
    this.isLoading = false;
    this.currentPrompt = "";
    this.promptType = "info";
    this.logs = [];
    this.state = "disconnected";
  }

  getInfo() {
    return {
      id: "pageoslogin",
      name: "PageOS 登录",
      color1: "#3498db",
      color2: "#2980b9",
      blocks: [
        {
          opcode: "connect",
          blockType: Scratch.BlockType.COMMAND,
          text: "连接 [URL]",
          arguments: {
            URL: {
              type: Scratch.ArgumentType.STRING,
              defaultValue: "ws://127.0.0.1:12801/ws",
            },
          },
        },
        {
          opcode: "onConnect",
          blockType: Scratch.BlockType.HAT,
          text: "当建立连接",
          isEdgeActivated: false,
        },
        {
          opcode: "isConnected",
          blockType: Scratch.BlockType.BOOLEAN,
          text: "已连接？",
        },
        {
          opcode: "onMessage",
          blockType: Scratch.BlockType.HAT,
          text: "当收到信息",
          isEdgeActivated: false,
        },
        {
          opcode: "onError",
          blockType: Scratch.BlockType.HAT,
          text: "当发生连接错误",
          isEdgeActivated: false,
        },
        {
          opcode: "onClose",
          blockType: Scratch.BlockType.HAT,
          text: "当连接关闭",
          isEdgeActivated: false,
        },
        {
          opcode: "sendAuthRequest",
          blockType: Scratch.BlockType.COMMAND,
          text: "发送验证请求 用户名 [USERNAME]",
          arguments: {
            USERNAME: {
              type: Scratch.ArgumentType.STRING,
              defaultValue: "user",
            },
          },
        },
        {
          opcode: "sendAuthResponse",
          blockType: Scratch.BlockType.COMMAND,
          text: "发送认证响应 信息 [RESPONSE]",
          arguments: {
            RESPONSE: {
              type: Scratch.ArgumentType.STRING,
              defaultValue: "",
            },
          },
        },
        {
          opcode: "sendStartSession",
          blockType: Scratch.BlockType.COMMAND,
          text: "发送启动会话响应 环境变量 [ENV] 启动命令 [CMD]",
          arguments: {
            ENV: {
              type: Scratch.ArgumentType.STRING,
              defaultValue: "LANG=zh_CN.UTF-8",
            },
            CMD: {
              type: Scratch.ArgumentType.STRING,
              defaultValue: "%SESSION_COMMAND%",
            },
          },
        },
        {
          opcode: "getSessionState",
          blockType: Scratch.BlockType.REPORTER,
          text: "获取会话状态",
        },
        {
          opcode: "isLoading",
          blockType: Scratch.BlockType.BOOLEAN,
          text: "正在加载中？",
        },
        {
          opcode: "getPrompt",
          blockType: Scratch.BlockType.REPORTER,
          text: "获取提示信息",
        },
        {
          opcode: "getPromptType",
          blockType: Scratch.BlockType.REPORTER,
          text: "获取提示信息类型",
        },
        {
          opcode: "getLogs",
          blockType: Scratch.BlockType.REPORTER,
          text: "获取日志信息",
        },
        {
          opcode: "clearLogs",
          blockType: Scratch.BlockType.COMMAND,
          text: "清空日志",
        },
      ],
    };
  }

  connect(args) {
    const url = args.URL;
    if (this.ws) {
      this.ws.close();
    }

    this.isLoading = true;
    this.logMessage(`正在连接到: ${url}`, "info");
    this.state = "connecting";

    this.ws = new WebSocket(url);

    this.ws.onopen = () => {
      this.connected = true;
      this.isLoading = false;
      this.state = "connected";
      this.logMessage("连接已建立", "info");
      Scratch.vm.runtime.startHats("pageoslogin_onConnect");
    };

    this.ws.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data);
        this.logMessage(`收到消息: ${JSON.stringify(msg)}`, "info");

        if (msg.type === "AUTH_MESSAGE") {
          this.currentPrompt = msg.message;
          this.promptType = msg.message_type.toLowerCase();
        }

        Scratch.vm.runtime.startHats("pageoslogin_onMessage");
      } catch (e) {
        this.logMessage(`消息解析错误: ${e}`, "error");
      }
    };

    this.ws.onerror = (error) => {
      this.isLoading = false;
      this.logMessage(`连接错误: ${error}`, "error");
      this.state = "error";
      Scratch.vm.runtime.startHats("pageoslogin_onError");
    };

    this.ws.onclose = () => {
      this.connected = false;
      this.isLoading = false;
      this.state = "disconnected";
      this.logMessage("连接已关闭", "warn");
      Scratch.vm.runtime.startHats("pageoslogin_onClose");
    };
  }

  sendAuthRequest(args) {
    if (!this.connected) return;

    const username = args.USERNAME;
    this.isLoading = true;
    this.logMessage(`发送验证请求: ${username}`, "info");
    this.state = "authenticating";

    this.ws.send(
      JSON.stringify({
        type: "AUTH_REQUEST",
        username: username,
      })
    );
  }

  sendAuthResponse(args) {
    if (!this.connected) return;

    const response = args.RESPONSE;
    this.isLoading = true;
    this.logMessage(`发送认证响应`, "info");

    this.ws.send(
      JSON.stringify({
        type: "AUTH_RESPONSE",
        response: response,
      })
    );
  }

  sendStartSession(args) {
    if (!this.connected) return;

    const env = args.ENV;
    const cmd = args.CMD;
    this.isLoading = true;
    this.logMessage(`发送启动会话请求`, "info");
    this.state = "starting_session";

    // 将环境变量和命令字符串转换为数组
    const envArray = env.split(",").map((item) => item.trim());
    const cmdArray = cmd
      .match(/[^\s"']+|"([^"]*)"|'([^']*)'/g)
      .map((arg) => arg.replace(/^["']|["']$/g, ""));

    this.ws.send(
      JSON.stringify({
        type: "START_SESSION",
        env: envArray,
        cmd: cmdArray,
      })
    );
  }

  isConnected() {
    return this.connected;
  }

  isLoading() {
    return this.isLoading;
  }

  getSessionState() {
    return this.state;
  }

  getPrompt() {
    return this.currentPrompt;
  }

  getPromptType() {
    return this.promptType;
  }

  getLogs() {
    return this.logs.map((log) => `${log.time}: ${log.message}`).join("\n");
  }

  clearLogs() {
    this.logs = [];
  }

  logMessage(message, type = "info") {
    const time = new Date().toLocaleTimeString();
    this.logs.push({ time, message, type });
    // 限制日志长度
    if (this.logs.length > 100) this.logs.shift();
  }
}

Scratch.extensions.register(new PageOSLoginExtension());
