# Start dev server
dev:
    cargo leptos watch

# Type-check server only (fast)
check-ssr:
    cargo check --features ssr

# Type-check WASM only (fast)
check-wasm:
    cargo check --features hydrate --target wasm32-unknown-unknown

# Check both targets
check: check-ssr check-wasm

# Run all tests
test:
    cargo test --features ssr

# Lint
lint:
    cargo clippy --features ssr -- -D warnings

# Format
fmt:
    cargo fmt
