use crate::error::Result;
use crate::models::{ChatCompletionRequest, ResponsesRequest};

/// Converts a Codex /responses request into an SGLang /chat/completions request.
pub fn convert(request: &ResponsesRequest) -> Result<ChatCompletionRequest> {
    let messages: Vec<_> = request
        .input
        .messages
        .iter()
        .map(convert_message)
        .collect::<Result<Vec<_>>>()?;

    // Pick max_tokens: explicit field on input > top-level field
    let max_tokens = request
        .input
        .explicit_max_tokens
        .or(request.top_max_tokens);

    // Pick temperature and top_p: explicit on input > top-level
    let temperature = request
        .input
        .explicit_temperature
        .or(request.temperature);
    let top_p = request.input.explicit_top_p.or(request.top_p);
    let stream = request.input.explicit_stream.or(request.stream);

    // Extract known fields from flattened extra
    let known = ["model", "max_tokens", "temperature", "top_p", "stream", "messages", "metadata"];
    let extra = if let Ok(obj) = serde_json::value::to_value(&request.extra) {
        if let Some(map) = obj.as_object() {
            map.iter()
                .filter(|(k, _)| !known.contains(&k.as_str()))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        } else {
            serde_json::Map::new()
        }
    } else {
        serde_json::Map::new()
    };

    Ok(ChatCompletionRequest {
        model: request.model.clone(),
        messages,
        max_completion_tokens: max_tokens.map(|t| t as u64),
        temperature,
        top_p,
        stream,
        extra,
    })
}

/// Converts a single message, mapping role:developer → role:system
fn convert_message(msg: &crate::models::ResponsesMessage) -> Result<crate::models::ChatMessage> {
    let role = match msg.role.as_str() {
        "developer" => "system",
        "user" => "user",
        "assistant" => "assistant",
        "system" => "system",
        other => {
            tracing::warn!("unknown role '{other}', forwarding as-is");
            other
        }
    };

    // Content in /responses can be string or array of content blocks
    // /chat/completions expects a string; flatten if needed
    let content = match &msg.content {
        serde_json::Value::String(s) if !s.is_empty() => {
            serde_json::Value::String(s.clone())
        }
        serde_json::Value::Array(blocks) if !blocks.is_empty() => {
            // Flatten text blocks into a single string
            let parts: Vec<String> = blocks
                .iter()
                .filter_map(|b| {
                    if let Some(text) = b.get("text").and_then(|t| t.as_str()) {
                        Some(text.to_string())
                    } else if b.is_string() {
                        b.as_str().map(|s| s.to_string())
                    } else {
                        None
                    }
                })
                .collect();
            if parts.is_empty() {
                msg.content.clone()
            } else {
                serde_json::Value::String(parts.join("\n\n"))
            }
        }
        serde_json::Value::String(s) => serde_json::Value::String(s.clone()),
        other => other.clone(),
    };

    Ok(crate::models::ChatMessage {
        role: role.to_string(),
        content,
    })
}
