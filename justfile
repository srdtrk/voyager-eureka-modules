# Lint the Rust code usind `cargo fmt` and `cargo clippy` commands
lint:
  @echo "Linting the Rust code..."
  cargo fmt --all -- --check
  cargo clippy

# Fix the Rust code using `cargo fmt` and `cargo clippy` commands
lint-fix:
  @echo "Fixing the Rust code..."
  cargo fmt --all
  cargo clippy --fix --allow-dirty --allow-staged

# Run the Rust tests using `cargo test` command.
test-cargo:
  cargo test --workspace --locked --all-features
