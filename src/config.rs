use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "codex-sglang-bridge")]
#[command(about = "Translates Codex /responses to SGLang /chat/completions")]
pub struct Config {
    /// Local port to listen on
    #[arg(long, default_value = "4000")]
    pub port: u16,

    /// SGLang server full URL including protocol, host, port, and endpoint
    /// (e.g. http://10.0.0.1:3000/v1/chat/completions)
    #[arg(long, env = "SGLANG_HOST")]
    pub sglang_host: String,

    /// Number of async worker threads for the Tokio runtime
    #[arg(long, default_value_t = 4)]
    pub worker_threads: usize,
}

impl Config {
    pub fn listen_addr(&self) -> String {
        format!("0.0.0.0:{}", self.port)
    }
}
