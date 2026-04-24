# codex-sglang-bridge

A lightweight Rust proxy that translates [OpenAI Codex `/responses`](https://platform.openai.com/docs/api-reference/responses) API requests into [SGLang `/chat/completions`](https://docs.sglang.ai/api/chat_completions.html) requests and forwards them.

Designed for developers who want to connect tools/apps using the Codex Responses format to SGLang backends (self-hosted or cloud) without writing custom adapters.

---

## Table of Contents

- [Features](#features)
- [Quick Start](#quick-start)
- [Configuration](#configuration)
- [Usage Examples](#usage-examples)
- [API Mapping](#api-mapping)
- [Build & Test](#build--test)
- [Runtime & Concurrency](#runtime--concurrency)
- [Troubleshooting](#troubleshooting)

---

## Features

- **Zero-dependency translation**: Converts Codex `responses` format → SGLang `chat/completions` format in-memory.
- **Streaming support**: Transparently proxies Server-Sent Events (SSE) streams.
- **Multi-threaded async runtime**: Configurable Tokio worker threads for high-concurrency workloads.
- **Observability**: Structured logging via `tracing` (supports `RUST_LOG`).
- **Small footprint**: Single static binary, no external services required.

---

## Quick Start

### Prerequisites

- Rust 1.70+ (`rustup` recommended)
- A running SGLang instance with `/v1/chat/completions` exposed

### 1. Clone and build

```bash
git clone https://github.com/your-org/codex-sglang-bridge.git
cd codex-sglang-bridge

# Build release binary
cargo build --release

# Verify tests pass
cargo test
```

### 2. Run

```bash
# Export SGLang endpoint via environment variable
export SGLANG_HOST="http://10.0.0.1:3000/v1/chat/completions"

# Start the bridge (default port 4000)
cargo run --release
```

The server will listen on `0.0.0.0:4000`.

---

## Configuration

| Variable / Flag | Description | Default | Example |
|---|---|---|---|
| `SGLANG_HOST` (env) or `--sglang-host` | Full upstream SGLang URL (incl. protocol, host, port, endpoint) | *(required)* | `http://10.0.0.1:3000/v1/chat/completions` |
| `--port` | Local listen port | `4000` | `8080` |
| `--worker-threads` | Tokio async runtime worker count | `4` | `16` |
| `RUST_LOG` | Log level | `info` | `debug` |

### Notes

- `SGLANG_HOST` **must** include the full path (e.g. `/v1/chat/completions`). The bridge forwards POST requests to this exact URL.
- The bridge listens on `0.0.0.0` by default. Change via `--port`.
- Worker threads control parallel task throughput; tune based on CPU cores and expected concurrency.

---

## Usage Examples

### Basic usage

```bash
SGLANG_HOST="http://10.0.0.1:3000/v1/chat/completions" \
  cargo run --release
```

### Custom port and thread pool

```bash
SGLANG_HOST="http://10.0.0.1:3000/v1/chat/completions" \
  cargo run --release -- --port 8080 --worker-threads 8
```

### Debug logging

```bash
RUST_LOG=debug SGLANG_HOST="http://10.0.0.1:3000/v1/chat/completions" \
  cargo run --release
```

### Docker (optional)

```dockerfile
FROM rust:1.80-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release
FROM debian:bookworm-slim
COPY --from=builder /app/target/release/codex-sglang-bridge /usr/local/bin/
CMD ["codex-sglang-bridge"]
```

---

## API Mapping

The bridge translates Codex `responses` fields → SGLang `chat/completions` fields.

### Message role mapping

| Codex role | SGLang role |
|---|---|
| `developer` | `system` |
| `user` | `user` |
| `assistant` | `assistant` |
| `system` | `system` |
| *other* | forwarded as-is (with warning) |

### Content flattening

- Codex `ContentBlock` arrays are joined with `\n\n` into a single string.
- Plain strings pass through unchanged.

### Parameter precedence

| SGLang parameter | Precedence order |
|---|---|
| `max_completion_tokens` | `input.explicit_max_tokens` → `top_max_tokens` |
| `temperature` | `input.explicit_temperature` → `temperature` |
| `top_p` | `input.explicit_top_p` → `top_p` |
| `stream` | `input.explicit_stream` → `stream` |

### Extra fields

- Any non-standard fields in `extra` are forwarded in the `extra` map on the SGLang request.
- Known fields (`model`, `max_tokens`, `temperature`, `top_p`, `stream`, `messages`, `metadata`) are excluded from `extra` to avoid duplication.

---

## Build & Test

| Command | Description |
|---|---|
| `cargo check` | Fast compilation check |
| `cargo build` | Debug build to `target/debug/` |
| `cargo build --release` | Optimized release binary |
| `cargo test` | Run unit tests (including concurrency regression) |
| `cargo test -- --nocapture` | Show test output |

---

## Runtime & Concurrency

- **Runtime**: Tokio multi-threaded runtime (`tokio::runtime::Builder::new_multi_thread()`).
- **Default workers**: `4` (configurable via `--worker-threads`).
- **HTTP client**: `reqwest::Client` reuses HTTP/2 connection pools automatically.
- **Streaming**: SSE body is forwarded via `axum::body::Body::from_stream()` without buffering the full response.

Concurrency regression test (`test_concurrent_conversion`) spawns **20 simultaneous requests** to verify thread safety. Run via `cargo test`.

---

## Troubleshooting

| Symptom | Cause / Fix |
|---|---|
| `error[E0063]: missing field \`listen_addr\`` | Stale `target/` after config changes. Run `cargo clean`. |
| `400 Bad Request` from bridge | Invalid Codex payload or missing required fields. Check `RUST_LOG=debug`. |
| `502 Bad Gateway` from bridge | SGLang unreachable or wrong `SGLANG_HOST`. Verify URL and network. |
| `upstream unreachable: connection refused` | SGLang not running or firewall blocking port. |
| Missing headers / CORS errors | Add `tower-http::cors::CorsLayer` to `server.rs` router layer stack. |
| High latency on streaming | Increase `--worker-threads` or tune Tokio `max_blocking_threads`. |

---

## License

MIT
