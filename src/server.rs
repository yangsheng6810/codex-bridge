use axum::response::Response;
use axum::routing::post;
use axum::Router;
use futures_util::stream::TryStreamExt;
use reqwest::Client;
use tower_http::trace::TraceLayer;
use tracing::{error, info, instrument};

use crate::converter::convert;
use crate::models::ResponsesRequest;

#[derive(Clone)]
pub struct AppState {
    pub client: Client,
    pub sglang_url: String,
    pub listen_addr: String,
}

pub async fn run_router(state: AppState) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let listen_addr = state.listen_addr.clone();
    let sglang_url = state.sglang_url.clone();
    let _client = state.client.clone();

    let app = Router::new()
        .route("/v1/responses", post(handle_responses))
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    info!("listening on {}", listen_addr);
    let listener = tokio::net::TcpListener::bind(&listen_addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

#[instrument(skip_all, fields(model, stream))]
async fn handle_responses(
    axum::extract::State(state): axum::extract::State<AppState>,
    body: axum::Json<ResponsesRequest>,
) -> Response {
    let is_stream = body.stream.unwrap_or(false);
    tracing::Span::current().record("model", &body.model);
    tracing::Span::current().record("stream", is_stream);

    info!("received /responses request");

    let chat_req = match convert(&body) {
        Ok(req) => req,
        Err(e) => {
            error!(error = %e, "conversion failed");
            return Response::builder()
                .status(400)
                .body(axum::body::Body::from(format!("{{\"error\": \"{e}\"}}")))
                .unwrap();
        }
    };

    let url = state.sglang_url;

    if is_stream {
        info!(url = %url, "forwarding streaming request");
        forward_streaming(&state.client, &url, &chat_req).await
    } else {
        info!(url = %url, "forwarding non-streaming request");
        forward_non_streaming(&state.client, &url, &chat_req).await
    }
}

async fn forward_non_streaming(
    client: &Client,
    url: &str,
    chat_req: &crate::models::ChatCompletionRequest,
) -> Response {
    let res = match client.post(url).json(chat_req).send().await {
        Ok(r) => r,
        Err(e) => {
            error!(error = %e, "forward request failed");
            return Response::builder()
                .status(502)
                .body(axum::body::Body::from(format!("{{\"error\": \"upstream unreachable: {e}\"}}")))
                .unwrap();
        }
    };

    let status = res.status();
    let body_bytes = match res.bytes().await {
        Ok(b) => b,
        Err(e) => {
            error!(error = %e, "read upstream body failed");
            return Response::builder()
                .status(502)
                .body(axum::body::Body::from("{}"))
                .unwrap();
        }
    };

    info!(status = status.as_u16(), "upstream response forwarded");

    Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(body_bytes.into())
        .unwrap()
}

async fn forward_streaming(
    client: &Client,
    url: &str,
    chat_req: &crate::models::ChatCompletionRequest,
) -> Response {
    let res = match client.post(url).json(chat_req).send().await {
        Ok(r) => r,
        Err(e) => {
            error!(error = %e, "forward request failed");
            return Response::builder()
                .status(502)
                .body(axum::body::Body::from(format!("{{\"error\": \"upstream unreachable: {e}\"}}")))
                .unwrap();
        }
    };

    let status = res.status();
    let content_type = res
        .headers()
        .get("content-type")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("text/event-stream")
        .to_owned();

    let body_stream = res.bytes_stream();
    let line_stream = body_stream.map_ok(|chunk| {
        String::from_utf8_lossy(&chunk).to_string()
    });

    let response_body = axum::body::Body::from_stream(line_stream);

    Response::builder()
        .status(status)
        .header("content-type", content_type)
        .header("cache-control", "no-cache")
        .header("connection", "keep-alive")
        .body(response_body)
        .unwrap()
}
