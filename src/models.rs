use serde::Deserialize;

// ── Codex /responses request ─────────────────────────

#[derive(Debug, Deserialize)]
pub struct ResponsesRequest {
    pub model: String,
    pub input: ResponsesInput,
    #[serde(rename = "max_tokens")]
    pub top_max_tokens: Option<u64>,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub stream: Option<bool>,
    pub metadata: Option<serde_json::Value>,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct ResponsesInput {
    #[serde(default)]
    pub messages: Vec<ResponsesMessage>,
    #[serde(default, rename = "model")]
    pub explicit_model: Option<String>,
    #[serde(default, rename = "max_tokens")]
    pub explicit_max_tokens: Option<u64>,
    #[serde(default, rename = "temperature")]
    pub explicit_temperature: Option<f64>,
    #[serde(default, rename = "top_p")]
    pub explicit_top_p: Option<f64>,
    #[serde(default, rename = "stream")]
    pub explicit_stream: Option<bool>,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct ResponsesMessage {
    pub role: String,
    pub content: serde_json::Value,
}

// ── SGLang /chat/completions request ─────────────────

#[derive(Debug, serde::Serialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(rename = "max_tokens", skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<u64>,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(flatten, skip_serializing_if = "serde_json::Map::is_empty")]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, serde::Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    #[serde(skip_serializing_if = "serde_json::Value::is_null")]
    pub content: serde_json::Value,
}

// ── SGLang /chat/completions response ────────────────

#[derive(Debug, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub model: String,
    pub created: u64,
    #[serde(default)]
    pub choices: Vec<Choice>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub index: u64,
    pub message: ChatMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
}

// ── Streaming delta ──────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ChatCompletionChunk {
    pub id: String,
    pub object: String,
    pub model: String,
    pub created: u64,
    #[serde(default)]
    pub choices: Vec<ChunkChoice>,
}

#[derive(Debug, Deserialize)]
pub struct ChunkChoice {
    pub index: u64,
    pub delta: ChunkDelta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChunkDelta {
    pub role: Option<String>,
    pub content: Option<String>,
}
