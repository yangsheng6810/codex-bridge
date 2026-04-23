# Repository Guidelines

## Project Overview

`codex-sglang-bridge` is a Rust binary that translates Codex `/responses` API requests into SGLang `/chat/completions` requests and proxies them. It runs as an HTTP server on port 4000 (configurable).

## Project Structure

```
src/
  main.rs          # Entry point: CLI parsing, tracing setup, server startup
  config.rs        # CLI arguments (port, SGLANG_URL)
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
| `cargo test` | Run all unit tests |

### Running Locally

```bash
SGLANG_URL=http://localhost:3000 cargo run
# Custom port:
SGLANG_URL=http://localhost:3000 cargo run -- --port 8080
```

## Coding Style

- **Edition**: Rust 2021, `serde` derive for all API structs.
- **Modules**: One public module per file. API shapes in `models.rs`, conversion in `converter.rs`.
- **Naming**: `snake_case` for functions/variables, `PascalCase` for types and modules.
- **Lints**: Default `cargo check` clippy level. Mark unused public structs/fields with `#[allow(dead_code)]` when they are part of the API contract.
- **Error handling**: Use `thiserror` enums with `?` propagation.

## Testing Guidelines

- Tests are inline in `src/main.rs` under `#[cfg(test)] mod tests`.
- Names follow `test_<module>_<description>` (e.g., `test_convert_basic`).
- Import via `crate::module_name`, not external crate names.
- Run with `cargo test`. No external services required.

## Commit & PR Guidelines

- **Commits**: Use conventional style (`feat:`, `fix:`, `refactor:`, `test:`, `chore:`). Keep the body concise; reference issues if applicable.
- **PRs**: Include a brief description of what changed and why. No screenshots needed.
- **CI**: Currently a `cargo test` + `cargo check` gate. Keep it minimal.

## Configuration Tips

| Env var / flag | Purpose |
|---|---|
| `SGLANG_URL` (env) or `--sglang-url` (flag) | Backend SGLang server URL (required) |
| `--port` (flag, default `4000`) | Local listen port |
| `RUST_LOG` | Tracing log level (e.g., `RUST_LOG=debug`) |

Run `cargo test` after any dependency change to verify compatibility.
