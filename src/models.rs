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
    #[allow(dead_code)]
    pub metadata: Option<serde_json::Value>,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

/// `ResponsesInput` can arrive as either a struct or a raw sequence of messages.
/// The OpenAI /responses API accepts `input` as a string, a message array, or a
/// full `ResponsesInput`-shaped object.
#[derive(Debug, Default)]
pub struct ResponsesInput {
    pub messages: Vec<ResponsesMessage>,
    #[allow(dead_code)]
    pub explicit_model: Option<String>,
    pub explicit_max_tokens: Option<u64>,
    pub explicit_temperature: Option<f64>,
    pub explicit_top_p: Option<f64>,
    pub explicit_stream: Option<bool>,
    #[allow(dead_code)]
    pub extra: serde_json::Value,
}

impl<'de> Deserialize<'de> for ResponsesInput {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;

        match value {
            // `input` is a struct → parse fields manually.
            serde_json::Value::Object(map) => {
                let messages = map
                    .get("messages")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| {
                                serde_json::from_value::<ResponsesMessage>(v.clone()).ok()
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                let explicit_model = map
                    .get("model")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let explicit_max_tokens = map
                    .get("max_tokens")
                    .and_then(|v| v.as_u64());
                let explicit_temperature = map
                    .get("temperature")
                    .and_then(|v| v.as_f64());
                let explicit_top_p = map.get("top_p").and_then(|v| v.as_f64());
                let explicit_stream = map.get("stream").and_then(|v| v.as_bool());

                let mut remaining = serde_json::Map::new();
                let known_keys = [
                    "messages", "model", "max_tokens", "temperature",
                    "top_p", "stream",
                ];
                for (k, v) in map {
                    if !known_keys.contains(&k.as_str()) {
                        remaining.insert(k, v);
                    }
                }

                Ok(ResponsesInput {
                    messages,
                    explicit_model,
                    explicit_max_tokens,
                    explicit_temperature,
                    explicit_top_p,
                    explicit_stream,
                    extra: serde_json::Value::Object(remaining),
                })
            }

            // `input` is a raw array of messages → wrap into a struct.
            serde_json::Value::Array(arr) => {
                let messages = arr
                    .into_iter()
                    .filter_map(|v| {
                        serde_json::from_value::<ResponsesMessage>(v).ok()
                    })
                    .collect();
                Ok(ResponsesInput {
                    messages,
                    ..Default::default()
                })
            }

            // `input` is a string (single user message) → wrap into a message.
            serde_json::Value::String(s) => Ok(ResponsesInput {
                messages: vec![ResponsesMessage {
                    role: "user".to_string(),
                    content: serde_json::Value::String(s),
                }],
                ..Default::default()
            }),

            _ => Err(serde::de::Error::custom(
                "input must be an object, array, or string",
            )),
        }
    }
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
#[allow(dead_code)]
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
#[allow(dead_code)]
pub struct Choice {
    pub index: u64,
    pub message: ChatMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Usage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
}

// ── Streaming delta ──────────────────────────────────

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ChatCompletionChunk {
    pub id: String,
    pub object: String,
    pub model: String,
    pub created: u64,
    #[serde(default)]
    pub choices: Vec<ChunkChoice>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ChunkChoice {
    pub index: u64,
    pub delta: ChunkDelta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ChunkDelta {
    pub role: Option<String>,
    pub content: Option<String>,
}
