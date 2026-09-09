# Database IDE

Milestone 1 backend foundation for managing saved PostgreSQL connections.

## Development

Install Rust, then run:

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo check
```

The desktop shell uses Tauri 2 and pnpm. After installing Node.js and Rust, run `pnpm install` followed by `pnpm dev` (or `pnpm tauri dev`). The native shell exposes typed connection CRUD and test commands backed by the Rust service; direct browser opening currently uses the UI prototype fallback.

The current milestone is intentionally backend-only. The future UI must use `SavedConnection.environment` to render production connections with a distinct warning indicator and must not connect to PostgreSQL directly.
