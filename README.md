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

The current milestone is intentionally backend-only. The future UI must use `SavedConnection.environment` to render production connections with a distinct warning indicator and must not connect to PostgreSQL directly.
