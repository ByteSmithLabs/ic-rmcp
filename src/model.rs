use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::borrow::Cow;

mod tool;

#[derive(Debug)]
pub struct JsonRpcVersion2_0;

impl Serialize for JsonRpcVersion2_0 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str("2.0")
    }
}

impl<'de> Deserialize<'de> for JsonRpcVersion2_0 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s == "2.0" {
            Ok(JsonRpcVersion2_0)
        } else {
            Err(serde::de::Error::custom(format!(
                "expected `{}`, got `{}`",
                "2.0", s
            )))
        }
    }
}

pub struct ProtocolVersion(Cow<'static, str>);

impl ProtocolVersion {
    pub const V_2025_03_26: Self = Self(Cow::Borrowed("2025-03-26"));
}

impl Serialize for ProtocolVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ProtocolVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        if s.as_str() == "2025-03-26" {
            return Ok(ProtocolVersion::V_2025_03_26);
        }

        Ok(ProtocolVersion(Cow::Owned(s)))
    }
}

#[derive(Debug, PartialEq)]
pub enum RequestId {
    Number(u32),
    String(String),
}

impl Serialize for RequestId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            RequestId::Number(n) => n.serialize(serializer),
            RequestId::String(s) => s.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for RequestId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value: Value = Deserialize::deserialize(deserializer)?;
        match value {
            Value::Number(n) => Ok(RequestId::Number(
                n.as_u64()
                    .ok_or(serde::de::Error::custom("Expect a positive integer"))?
                    as u32,
            )),
            Value::String(s) => Ok(RequestId::String(s.into())),
            _ => Err(serde::de::Error::custom(
                "Expect a positive interger or a string",
            )),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Request {
    pub method: String,
    pub params: serde_json::Map<String, Value>,
}

pub struct RequestOptionalParam {
    pub method: String,
    pub params: Option<serde_json::Map<String, Value>>,
}

pub struct RequestNoParam {
    pub method: String,
}

#[derive(Serialize, Deserialize)]
pub struct Notification {
    pub method: String,
    pub params: serde_json::Map<String, Value>,
}

pub struct NotificationNoParam {
    pub method: String,
}

#[derive(Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: JsonRpcVersion2_0,
    pub id: RequestId,
    #[serde(flatten)]
    pub request: Request,
}

#[derive(Serialize, Deserialize)]
pub struct JsonRpcResultResponse {
    pub jsonrpc: JsonRpcVersion2_0,
    pub id: RequestId,
    pub result: serde_json::Map<String, Value>,
}

#[derive(Serialize, Deserialize)]
pub struct JsonRpcErrorResponse {
    pub jsonrpc: JsonRpcVersion2_0,
    pub id: RequestId,
    pub error: ErrorData,
}

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct ErrorCode(pub i32);

#[derive(Serialize, Deserialize)]
pub struct ErrorData {
    pub code: ErrorCode,

    pub message: Cow<'static, str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl ErrorData {
    pub fn new(
        code: ErrorCode,
        message: impl Into<Cow<'static, str>>,
        data: Option<Value>,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            data,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct JsonRpcNotification {
    pub jsonrpc: JsonRpcVersion2_0,
    #[serde(flatten)]
    pub notification: Notification,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcMessage {
    Request(JsonRpcRequest),
    Response(JsonRpcResultResponse),
    Notification(JsonRpcNotification),
    BatchRequest(Vec<JsonRpcBatchIngressItem>),
    BatchResponse(Vec<JsonRpcBatchEgressItem>),
    Error(JsonRpcErrorResponse),
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcBatchIngressItem {
    Request(JsonRpcRequest),
    Notification(JsonRpcNotification),
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcBatchEgressItem {
    Response(JsonRpcResultResponse),
    Error(JsonRpcErrorResponse),
}

impl JsonRpcBatchIngressItem {
    pub fn into_non_batch_message<Resp>(self) -> JsonRpcMessage {
        match self {
            JsonRpcBatchIngressItem::Request(r) => JsonRpcMessage::Request(r),
            JsonRpcBatchIngressItem::Notification(n) => JsonRpcMessage::Notification(n),
        }
    }
}

impl JsonRpcBatchEgressItem {
    pub fn into_non_batch_message<Req, Not>(self) -> JsonRpcMessage {
        match self {
            JsonRpcBatchEgressItem::Response(r) => JsonRpcMessage::Response(r),
            JsonRpcBatchEgressItem::Error(e) => JsonRpcMessage::Error(e),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Implementation {
    pub name: String,
    pub version: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeRequestParam {
    pub protocol_version: ProtocolVersion,
    pub client_info: Implementation,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolsCapability {}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerCapabilities {
    pub tools: Option<ToolsCapability>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResult {
    pub protocol_version: ProtocolVersion,
    pub capabilities: ServerCapabilities,
    pub server_info: Implementation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedRequestParam {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}