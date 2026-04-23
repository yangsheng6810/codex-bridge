use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "codex-sglang-bridge")]
#[command(about = "Translates Codex /responses to SGLang /chat/completions")]
pub struct Config {
    /// Local port to listen on
    #[arg(long, default_value = "4000")]
    pub port: u16,

    /// SGLang server URL (e.g. http://10.0.0.1:3000)
    #[arg(long, env = "SGLANG_URL")]
    pub sglang_url: String,
}

impl Config {
    pub fn listen_addr(&self) -> String {
        format!("0.0.0.0:{}", self.port)
    }
}
