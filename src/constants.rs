//! Constants for DingTalk Stream SDK

/// The DingTalk gateway URL for opening connections
pub const GATEWAY_URL: &str = "https://api.dingtalk.com/v1.0/gateway/connections/open";

/// The DingTalk API endpoint for getting access tokens
pub const GET_TOKEN_URL: &str = "https://api.dingtalk.com/v1.0/oauth2/accessToken";

/// The topic for robot message callbacks
pub const TOPIC_ROBOT: &str = "/v1.0/im/bot/messages/get";

/// The topic for robot delegate message callbacks
pub const TOPIC_ROBOT_DELEGATE: &str = "/v1.0/im/bot/messages/delegate";

/// The topic for card callback
pub const TOPIC_CARD: &str = "/v1.0/card/instances/callback";

/// The topic for AI Graph API plugin message callbacks
pub const TOPIC_AI_GRAPH_API: &str = "/v1.0/graph/api/invoke";

/// The default DingTalk OpenAPI endpoint
pub const DEFAULT_OPENAPI_ENDPOINT: &str = "https://api.dingtalk.com";

/// SDK version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");