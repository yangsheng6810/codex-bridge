use thiserror::Error;

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum BridgeError {
    #[error("invalid request body: {0}")]
    InvalidBody(String),

    #[error("forward request failed: {0}")]
    ForwardError(#[from] reqwest::Error),

    #[error("upstream returned status {0}: {1}")]
    UpstreamError(u16, String),

    #[error("conversion failed: {0}")]
    ConversionError(String),
}

pub type Result<T> = std::result::Result<T, BridgeError>;
