# DingTalk Stream SDK for Rust

钉钉 Stream SDK 的 Rust 实现，参考了官方 Node.js / Python SDK 的能力模型。

## 功能特性

- WebSocket 长连接与自动重连
- `CALLBACK` / `EVENT` / `SYSTEM` 三类消息处理
- 自动 ACK 回执
- 回调会话 `sessionWebhook` 异步回复能力
- 机器人主动发消息（私聊 / 群聊）
- 文件与图片下载、媒体上传
- Access Token 自动缓存与续期
- 生命周期事件监听（连接、收发、心跳、断开）

## 安装

```toml
[dependencies]
dingtalk-stream-sdk = "0.1"
```

如需使用系统根证书池，可显式启用：

```toml
[dependencies]
dingtalk-stream-sdk = { version = "0.1", features = ["rustls-tls-native-roots"] }
```

## 环境变量

- `DINGTALK_CLIENT_ID`
- `DINGTALK_CLIENT_SECRET`

## 快速开始

```rust
use async_trait::async_trait;
use dingtalk_stream::frames::down_message::callback_message::{CallbackMessage, MessagePayload};
use dingtalk_stream::frames::down_message::MessageTopic;
use dingtalk_stream::frames::up_message::callback_message::WebhookMessage;
use dingtalk_stream::handlers::{CallbackHandler, Error, ErrorCode, Resp};
use dingtalk_stream::{Credential, DingTalkStream, TOPIC_ROBOT};
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

struct RobotHandler(MessageTopic);

#[async_trait]
impl CallbackHandler for RobotHandler {
    async fn process(
        &self,
        _client: &DingTalkStream,
        message: &CallbackMessage,
        _cb_webhook_msg_sender: Option<Sender<WebhookMessage>>,
    ) -> Result<Resp, Error> {
        if let Some(data) = &message.data {
            if let Some(MessagePayload::Text { text }) = &data.payload {
                return Ok(Resp::Text(format!("echo: {}", text.content)));
            }
        }
        Err(Error {
            msg: "unsupported message".to_string(),
            code: ErrorCode::BadRequest,
        })
    }

    fn topic(&self) -> &MessageTopic {
        &self.0
    }
}

#[tokio::main]
async fn main() -> dingtalk_stream::Result<()> {
    tracing_subscriber::fmt::init();

    let client = Arc::new(
        DingTalkStream::new(Credential::from_env())
            .register_callback_handler(Arc::new(RobotHandler(MessageTopic::Callback(
                TOPIC_ROBOT.to_string(),
            ))))
            .await,
    );

    let (_client, join_handle) = client.start().await?;
    join_handle.await??;
    Ok(())
}
```

完整示例见 `examples/hello.rs`。

## ClientConfig

```rust
use dingtalk_stream::ClientConfig;
use std::time::Duration;

let config = ClientConfig {
    auto_reconnect: true,
    ua: "my-bot/1.0".to_string(),
    reconnect_interval: Duration::from_secs(10),
    keep_alive_interval: Duration::from_secs(60),
};
```

## 主题常量

- `TOPIC_ROBOT`: `/v1.0/im/bot/messages/get`
- `TOPIC_ROBOT_DELEGATE`: `/v1.0/im/bot/messages/delegate`
- `TOPIC_CARD`: `/v1.0/card/instances/callback`

## 主动发机器人消息

```rust
use dingtalk_stream::frames::up_message::robot_message::{RobotMessage, RobotPrivateMessage};

client
    .send_message(
        RobotMessage::from(RobotPrivateMessage {
            user_ids: vec!["manager_userid".into()],
            content: "hello".into(),
        })
        .with_cb(|result| {
            println!("send result: {result:?}");
        }),
    )
    .await?;
```

## 下载回调中的文件/图片

`PayloadPicture` 和 `PayloadFile` 实现了 `DingtalkResource`，可在回调中直接下载：

```rust
use dingtalk_stream::client::DingtalkResource;

let (path, _bytes_or_image) = picture_payload.fetch(client, "/tmp".into()).await?;
println!("saved to: {}", path.display());
```

## 上传媒体

```rust
use dingtalk_stream::client::{DingTalkMedia, MediaImage};
use std::path::PathBuf;

let media = MediaImage::from(PathBuf::from("./test_resources/img.png"));
let result = media.upload(client).await?;
println!("upload: errcode={}, errmsg={}", result.errcode, result.errmsg);
```

## 生命周期监听

可实现 `LifecycleListener` 监听 `Start`、`Connected`、`WebsocketRead`、`Disconnected` 等事件，
并通过 `register_lifecycle_listener` 注入。

## License

MIT
