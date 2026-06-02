# Rust 1.95 + tokio + rmcp Backend Skill

Tech-stack conventions for Rust 1.95 (edition 2024) / tokio async runtime / rmcp MCP server / MongoDB driver.

---

## Stack Overview

- **Language:** Rust 1.95, edition 2024
- **Runtime:** tokio 1.x (multi-thread)
- **MCP framework:** rmcp 1.6 (Model Context Protocol, stdio transport)
- **CLI:** clap 4.x (derive macros)
- **HTTP client:** reqwest 0.12 (rustls-tls, JSON)
- **MongoDB:** mongodb driver 3.6 (rustls-tls, bson-2)
- **Config:** serde_yaml_ng (YAML), serde / serde_json
- **Auth:** OAuth2/PKCE flow via browser (tokio TCP listener for callback)
- **Error handling:** anyhow (application), thiserror (library errors)
- **Tracing:** tracing + tracing-subscriber (env-filter, plain text to stderr)
- **Build:** cargo (release: lto=true, codegen-units=1, strip=true)

---

## Module Layout

```
src/
├── main.rs           # CLI entry, clap, #[tokio::main]
├── lib.rs            # Public re-exports
├── server.rs         # ToolsServer — rmcp ServerHandler, tool registration
├── state.rs          # AppState — shared across tools via Arc
├── auth/
│   ├── mod.rs
│   ├── login.rs      # OAuth2/PKCE browser flow + session validation
│   └── token.rs      # StoredTokens, load/save/delete (OS config dir)
├── connections/
│   ├── mod.rs
│   ├── config.rs     # YAML config load, default path, validation
│   └── resolver.rs   # Connection pool resolution
├── tools/
│   ├── mod.rs
│   ├── error.rs      # ToolError (thiserror)
│   └── <tool>.rs     # One file per MCP tool
└── util/
    ├── mod.rs
    └── format.rs
```

---

## Tool Registration Pattern

Each tool module exposes a `*_route()` function using rmcp macros and is registered in `ToolsServer::new()`.

```rust
// tools/my_tool.rs
#[tool(tool_box)]
impl ToolsServer {
    #[tool(description = "Short tool description.")]
    async fn my_tool(&self, #[tool(aggr)] params: MyParams) -> CallToolResult {
        run_with_timeout(LIGHTWEIGHT_TOOL_TIMEOUT, async {
            // ... business logic returning Result<T, ToolError>
        }).await
    }
}

pub fn my_tool_route() -> ToolRoute<ToolsServer> {
    <ToolsServer as MyTool>::my_tool_router()
}

// server.rs — register in ToolsServer::new()
pub fn new(state: Arc<AppState>) -> Self {
    Self::empty(state)
        .with_route(my_tool::my_tool_route())
}
```

---

## Error Handling

- **Application errors:** `anyhow::Result<T>` in `main.rs` and bin entrypoints
- **Library errors:** `thiserror`-derived enums in domain modules
- **Tool errors:** `ToolError` (thiserror) → surfaced as `error_result()` via `run_with_timeout`
- **Never panic in tool handlers** — return `CallToolResult::error`, not `unwrap()` / `expect()`
- **`?` propagation** within async tool bodies is fine; the timeout wrapper catches it

---

## Async Rules

- `#[tokio::main]` only at the binary entrypoint
- Shared state is `Arc<AppState>`; cloned cheaply per request
- All tool I/O runs under `run_with_timeout(LIGHTWEIGHT_TOOL_TIMEOUT, ...)` (30 s)
- No blocking I/O on the async executor — use `tokio::task::spawn_blocking` if needed
- Prefer `tokio::sync` primitives over `std::sync` in async contexts

---

## Auth / Credentials

- OAuth2/PKCE: `auth::login::run_login()` opens a browser and binds a random TCP port for the callback
- Tokens (`access_token`, `id_token`, `refresh_token`) stored as JSON in `directories::ProjectDirs` config dir
- Every server startup validates the stored session via `login::validate_session()`
- `--login` / `--logout` CLI flags manage the credential lifecycle
- Auth endpoint: `https://auth.studio3t.com`
- **Never log token values** — log only structured events with user_id or error message

---

## Observability

- All output goes to **stderr**; **stdout is reserved for MCP JSON-RPC framing**
- Controlled by `RUST_LOG` env var (default `warn`; use `info` for verbose)
- Use `tracing::info!` / `tracing::error!` with structured key-value fields
- `tracing_subscriber::fmt().with_ansi(false)` — no ANSI escape codes

```rust
tracing::info!(connection_id = %id, database_count = count, "databases listed");
tracing::error!(error = %e, "failed to resolve connection");
```

---

## Testing

- Unit tests: pure domain logic, no tokio runtime where avoidable; use `#[test]`
- Async tests: `#[tokio::test]` — keep them focused, no real network calls
- Integration tests: `cargo test` — mock I/O or use test fixtures for YAML config
- Test names: behaviour claims (`given_valid_config_resolves_connections`)
- Config tests: always cover `NotFound`, malformed YAML, invalid id patterns, env-var expansion

---

## Build & Verify

```bash
cargo build
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release      # binary → target/release/stt-mcp
```

All code must pass `clippy -- -D warnings`. No exceptions.

---

## Secrets / Security

- Connection URIs may contain embedded credentials — **never log them**
- `env:VAR_NAME` prefix in config is resolved at load time; raw value is not persisted
- Config file is user-local; no credentials committed to the repo
- `Cargo.lock` is committed (binary crate — ensures reproducible builds)

---

## Common Pitfalls

- **Logging to stdout:** Breaks MCP stdio framing silently. Always use `tracing` / `eprintln!` (stderr).
- **Blocking in async:** `mongodb` driver and `reqwest` are async-safe; raw `std::fs` is not — wrap with `spawn_blocking`.
- **Panics in tool handlers:** Must not reach the MCP client. Return `error_result()` instead.
- **Clippy warnings:** They are errors in CI. Fix before pushing.
- **`unwrap()` on `Option` from config:** Validate at load time with proper `ConfigError` variants.

---

## See Also

- `README.md` — CLI usage, smoke tests, MCP client registration
- `Cargo.toml` — locked dependency versions