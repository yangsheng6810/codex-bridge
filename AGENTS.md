# Repository Guidelines

## Project Overview

`codex-sglang-bridge` is a Rust binary that translates Codex `/responses` API requests into SGLang `/chat/completions` requests and proxies them. It runs as an HTTP server with a multi-threaded Tokio runtime for concurrent request handling.

## Project Structure

```
src/
  main.rs          # Entry point: CLI parsing, multi-thread runtime, server startup
  config.rs        # CLI arguments (port, SGLANG_HOST, worker_threads)
  server.rs        # Axum router, request handlers, upstream proxy
  converter.rs     # /responses → /chat/completions transformation
  models.rs        # All request/response structs (serde Serialize/Deserialize)
  error.rs         # Custom error types
Cargo.toml         # Dependencies
```

Tests live inline in `src/main.rs` under `#[cfg(test)]`.

## Build, Test, and Development Commands

| Command | Description |
|---|---|
| `cargo check` | Fast compile-check without building |
| `cargo build` | Debug binary to `target/debug/` |
| `cargo build --release` | Optimized release binary |
| `cargo test` | Run all unit tests (including concurrency regression tests) |

### Running Locally

```bash
# Default config (port 4000, http://localhost:8000/v1/chat/completions)
SGLANG_HOST="http://localhost:8000/v1/chat/completions" cargo run

# Custom port and worker threads
SGLANG_HOST="http://10.0.0.1:3000/v1/chat/completions" \
  cargo run -- --port 8080 --worker-threads 8
```

## Configuration

| Env var / flag | Purpose | Example |
|---|---|---|
| `SGLANG_HOST` (env) or `--sglang-host` (flag) | Full SGLang URL with protocol, host, port, and endpoint | `http://10.0.0.1:3000/v1/chat/completions` |
| `--port` (flag, default `4000`) | Local listen port | `8080` |
| `--worker-threads` (flag, default `4`) | Tokio multi-thread runtime worker count | `8` |
| `RUST_LOG` | Tracing log level (e.g., `RUST_LOG=debug`) | `debug` |

> **Note**: `SGLANG_HOST` must include the full path (e.g., `/v1/chat/completions`). The bridge forwards to this exact endpoint.

## Concurrent Runtime

The server uses `tokio::runtime::Builder::new_multi_thread()` with configurable `worker_threads`. This allows synchronous blocking operations (e.g., large JSON serialization) to be offloaded to dedicated threads without stalling the async event loop.

- **Default**: 4 worker threads
- **Tuning**: Set via `--worker-threads` CLI flag
- **Concurrency model**: Axum dispatches each request independently; `reqwest::Client` reuses HTTP/2 connection pools
- **Regression tests**: `test_concurrent_conversion` spawns 20 concurrent tasks to verify no race conditions

## Coding Style

- **Edition**: Rust 2021, `serde` derive for all API structs.
- **Modules**: One public module per file. API shapes in `models.rs`, conversion in `converter.rs`.
- **Naming**: `snake_case` for functions/variables, `PascalCase` for types and modules.
- **Lints**: Default `cargo check` clippy level. Mark unused public structs/fields with `#[allow(dead_code)]` when they are part of the API contract.
- **Error handling**: Use `thiserror` enums with `?` propagation.

## Testing Guidelines

- Tests are inline in `src/main.rs` under `#[cfg(test)]`.
- Names follow `test_<module>_<description>` (e.g., `test_convert_basic`).
- Import via `crate::module_name`, not external crate names.
- `#[tokio::test]` for async tests.
- Concurrency regression tests use `tokio::spawn` + `join_all` to simulate concurrent request handling.
- Run with `cargo test`. No external services required.

## Commit & PR Guidelines

- **Commits**: Use conventional style (`feat:`, `fix:`, `refactor:`, `test:`, `chore:`). Keep the body concise; reference issues if applicable.
- **PRs**: Include a brief description of what changed and why. No screenshots needed.
- **CI**: Currently a `cargo test` + `cargo check` gate. Keep it minimal.
