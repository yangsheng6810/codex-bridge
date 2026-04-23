mod config;
mod converter;
mod error;
mod models;
mod server;

use clap::Parser;
use config::Config;
use server::{run_router, AppState};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = Config::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    info!(sglang_url = %config.sglang_url, "starting bridge server");

    let state = AppState {
        client: reqwest::Client::new(),
        sglang_url: config.sglang_url,
    };

    run_router(state).await
}

#[cfg(test)]
mod tests {
    use crate::converter::convert;
    use crate::models::{ResponsesInput, ResponsesMessage, ResponsesRequest};

    fn make_request(model: &str, messages: Vec<(String, String)>, stream: bool) -> ResponsesRequest {
        ResponsesRequest {
            model: model.to_string(),
            input: ResponsesInput {
                messages: messages
                    .into_iter()
                    .map(|(role, content)| ResponsesMessage {
                        role,
                        content: serde_json::json!(content),
                    })
                    .collect(),
                explicit_model: None,
                explicit_max_tokens: None,
                explicit_temperature: None,
                explicit_top_p: None,
                explicit_stream: Some(stream),
                extra: serde_json::json!({}),
            },
            top_max_tokens: None,
            temperature: None,
            top_p: None,
            stream: Some(stream),
            metadata: None,
            extra: serde_json::json!({}),
        }
    }

    #[test]
    fn test_convert_basic() {
        let req = make_request(
            "codex-model",
            vec![
                ("developer".to_string(), "You are helpful.".to_string()),
                ("user".to_string(), "Hello!".to_string()),
            ],
            false,
        );
        let converted = convert(&req).unwrap();
        assert_eq!(converted.model, "codex-model");
        assert_eq!(converted.messages.len(), 2);
        assert_eq!(converted.messages[0].role, "system"); // developer -> system
        assert_eq!(converted.messages[1].role, "user");
        assert!(!converted.stream.unwrap());
    }

    #[test]
    fn test_convert_with_top_level_fields() {
        let mut req = make_request(
            "codex-model",
            vec![("user".to_string(), "Hi".to_string())],
            true,
        );
        req.temperature = Some(0.7);
        req.top_max_tokens = Some(100);

        let converted = convert(&req).unwrap();
        assert_eq!(converted.temperature, Some(0.7));
        assert_eq!(converted.max_completion_tokens, Some(100));
    }
}
