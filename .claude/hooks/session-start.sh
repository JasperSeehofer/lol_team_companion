#!/bin/bash
# SessionStart hook for Claude Code cloud sessions.
# Installs what `cargo leptos`, clippy/fmt, unit tests and the Playwright e2e
# suite need. Idempotent: every step checks first and is a no-op when done.
set -euo pipefail

if [ "${CLAUDE_CODE_REMOTE:-}" != "true" ]; then
  exit 0
fi

PROJECT_DIR="${CLAUDE_PROJECT_DIR:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$PROJECT_DIR"

CARGO_LEPTOS_VERSION="0.3.11"
PW_COMPAT_DIR="${PW_COMPAT_DIR:-/root/.cache/pw-browsers-compat}"
PW_SYSTEM_DIR="/opt/pw-browsers"

log() { echo "[session-start] $*"; }

# 1. Rust toolchain from rust-toolchain.toml (stable + wasm32 + clippy + rustfmt).
#    `rustup show` installs whatever the toolchain file asks for.
if command -v rustup >/dev/null 2>&1; then
  rustup show active-toolchain >/dev/null 2>&1 || true
  rustup component add clippy rustfmt >/dev/null 2>&1 || true
  rustup target add wasm32-unknown-unknown >/dev/null 2>&1 || true
fi

# 2. cargo-leptos, prebuilt from the GitHub release (no compile).
if ! cargo leptos --version 2>/dev/null | grep -q "$CARGO_LEPTOS_VERSION"; then
  log "installing cargo-leptos $CARGO_LEPTOS_VERSION"
  tmp="$(mktemp -d)"
  curl -sSfL -o "$tmp/cl.tar.gz" \
    "https://github.com/leptos-rs/cargo-leptos/releases/download/v${CARGO_LEPTOS_VERSION}/cargo-leptos-x86_64-unknown-linux-gnu.tar.gz"
  tar -xzf "$tmp/cl.tar.gz" -C "$tmp"
  mkdir -p "$HOME/.cargo/bin"
  install -m 0755 "$tmp/cargo-leptos-x86_64-unknown-linux-gnu/cargo-leptos" "$HOME/.cargo/bin/cargo-leptos"
  rm -rf "$tmp"
fi

# 3. .env with a dummy Riot key (real keys come from the environment only).
if [ ! -f .env ] && [ -f .env.example ]; then
  log "creating .env from .env.example with a dummy RIOT_API_KEY"
  sed 's/^RIOT_API_KEY=.*/RIOT_API_KEY=RGAPI-dummy/' .env.example > .env
fi

# 4. e2e dependencies.
if [ -f e2e/package-lock.json ] && [ ! -d e2e/node_modules/@playwright/test ]; then
  log "installing e2e dependencies"
  (cd e2e && npm ci --no-audit --no-fund)
fi

# 5. Playwright browser compat dir. The image ships chromium rev 1194 under
#    /opt/pw-browsers; Playwright 1.58 asks for rev 1208. Symlink one as the other.
if [ -d "$PW_SYSTEM_DIR/chromium-1194" ] && [ ! -e "$PW_COMPAT_DIR/chromium-1208/chrome-linux64" ]; then
  log "creating Playwright compat dir at $PW_COMPAT_DIR"
  mkdir -p "$PW_COMPAT_DIR/chromium-1208" \
           "$PW_COMPAT_DIR/chromium_headless_shell-1208/chrome-headless-shell-linux64"
  ln -sfn "$PW_SYSTEM_DIR/chromium-1194/chrome-linux" "$PW_COMPAT_DIR/chromium-1208/chrome-linux64"
  ln -sfn "$PW_SYSTEM_DIR/chromium_headless_shell-1194/chrome-linux/headless_shell" \
          "$PW_COMPAT_DIR/chromium_headless_shell-1208/chrome-headless-shell-linux64/chrome-headless-shell"
  for d in chromium-1208 chromium_headless_shell-1208; do
    touch "$PW_COMPAT_DIR/$d/INSTALLATION_COMPLETE" "$PW_COMPAT_DIR/$d/DEPENDENCIES_VALIDATED"
  done
  if [ -d "$PW_SYSTEM_DIR/ffmpeg-1011" ]; then
    ln -sfn "$PW_SYSTEM_DIR/ffmpeg-1011" "$PW_COMPAT_DIR/ffmpeg-1011"
  fi
fi

# 6. Session env vars for e2e and agent-browser.
if [ -n "${CLAUDE_ENV_FILE:-}" ]; then
  grep -q PLAYWRIGHT_BROWSERS_PATH "$CLAUDE_ENV_FILE" 2>/dev/null || {
    echo "export PLAYWRIGHT_BROWSERS_PATH=$PW_COMPAT_DIR" >> "$CLAUDE_ENV_FILE"
    echo "export AGENT_BROWSER_EXECUTABLE_PATH=$PW_SYSTEM_DIR/chromium" >> "$CLAUDE_ENV_FILE"
  }
fi

log "done"
