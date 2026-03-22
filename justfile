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

# Quick health check against the running dev server
health:
    ./scripts/health.sh

# Curl-based API smoke tests (requires running server + optional credentials)
# Usage: just api-test [base_url] [email] [password]
api-test *ARGS:
    ./scripts/api_test.sh {{ARGS}}

# Install Playwright and browsers (run once)
e2e-install:
    cd e2e && npm install && npx playwright install chromium

# Run Playwright e2e tests (requires running server)
e2e:
    cd e2e && npx playwright test

# Run e2e tests in headed mode (shows browser)
e2e-headed:
    cd e2e && npx playwright test --headed

# Run e2e with interactive UI
e2e-ui:
    cd e2e && npx playwright test --ui

# Full validation: check both targets + test + lint + format check
verify: check test lint
    cargo fmt --check

# Pipeline flow test: draft → game-plan → post-game linking (requires running server)
flow-test:
    ./scripts/flow_test.sh

# Runtime smoke tests (requires running server)
smoke: health api-test flow-test

# Wait for dev server to be ready (max timeout_seconds)
wait-for-server seconds="60":
    ./scripts/wait_for_server.sh {{seconds}}

# Check both targets + e2e (requires running server)
full-check: verify e2e
