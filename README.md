# DingTalk Stream SDK for Rust

钉钉流式 SDK 的 Rust 实现，基于 Node.js 和 Python 版本的官方 SDK。

## 功能特性

- WebSocket 长连接管理
- 自动重连
- 事件和回调消息处理
- 消息 ACK 确认
- 机器人消息处理
- AI Graph API 支持
- 卡片回调处理
- Access Token 管理

## 安装

```toml
# Cargo.toml
[dependencies]
dingtalk-stream-sdk = "0.1"
```

## 快速开始

```rust
use dingtalk_stream::{
    Credential, DingTalkStream, CallbackHandler, CallbackMessage,
    TOPIC_ROBOT,
};
use async_trait::async_trait;
use std::env;

/// 自定义机器人消息处理器
struct RobotHandler;

#[async_trait]
impl CallbackHandler for RobotHandler {
    async fn process(&self, message: &CallbackMessage) -> (i32, String) {
        // 从消息中提取文本
        if let Some(data) = &message.data {
            if let Some(text_obj) = data.get("text") {
                if let Some(content) = text_obj.get("content").and_then(|v| v.as_str()) {
                    println!("收到消息: {}", content);
                    return (200, format!("回复: {}", content));
                }
            }
        }
        (404, "not implement".to_string())
    }

    fn topic(&self) -> &str {
        TOPIC_ROBOT
    }
}

#[tokio::main]
async fn main() {
    // 初始化日志
    tracing_subscriber::fmt::init();

    // 从环境变量获取凭证
    let client_id = env::var("DINGTALK_CLIENT_ID").expect("请设置 DINGTALK_CLIENT_ID");
    let client_secret = env::var("DINGTALK_CLIENT_SECRET").expect("请设置 DINGTALK_CLIENT_SECRET");

    // 创建凭证
    let credential = Credential::new(client_id, client_secret);

    // 创建客户端（开启调试模式）
    let client = DingTalkStream::new(credential).with_debug(true);

    // 注册机器人消息处理器
    client.register_callback_handler(TOPIC_ROBOT, RobotHandler);

    // 启动客户端（会自动重连）
    client.start_forever().await;
}
```

## 环境变量

- `DINGTALK_CLIENT_ID`: 钉钉应用 Client ID
- `DINGTALK_CLIENT_SECRET`: 钉钉应用 Client Secret

## 配置选项

```rust
use dingtalk_stream::{Credential, ClientConfig, DingTalkStream};

let config = ClientConfig {
    auto_reconnect: true,      // 自动重连
    keep_alive: true,          // 保持连接
    debug: true,               // 调试模式
    reconnect_interval: 10,    // 重连间隔（秒）
    keep_alive_interval: 60,   // 心跳间隔（秒）
    ..Default::default()
};

let client = DingTalkStream::with_config(credential, config);
```

## 消息类型

### 回调消息 (CALLBACK)

- 机器人消息: `/v1.0/im/bot/messages/get`
- 卡片回调: `/v1.0/card/instances/callback`
- AI Graph API: `/v1.0/graph/api/invoke`

### 事件消息 (EVENT)

- 默认订阅: `*` (所有事件)

### 系统消息 (SYSTEM)

- CONNECTED: 连接建立
- REGISTERED: 注册成功
- disconnect: 断开连接
- KEEPALIVE: 心跳
- ping: ping 请求

## 发送消息响应

```rust
// 发送回调响应（避免服务端重试）
client.socket_callback_response(&message_id, result).await;

// 发送 Graph API 响应
client.send_graph_api_response(&message_id, response).await;
```

## 获取 Access Token

```rust
let token = client.get_access_token().await?;
```

## 示例

查看 `examples/hello.rs` 获取完整示例。

## 依赖

- tokio (full)
- tokio-tungstenite
- reqwest
- serde / serde_json
- futures-util
- parking_lot
- async-trait
- tracing

## 参考

- [钉钉开放平台文档](https://open.dingtalk.com/document/dingstart/start-overview)
- [Node.js SDK](https://github.com/open-dingtalk/dingtalk-stream-sdk-nodejs)
- [Python SDK](https://github.com/open-dingtalk/dingtalk-stream-sdk-python)

## License

MIT