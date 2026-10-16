# syntax=docker/dockerfile:1.7
#
# Companion container image (M8).
#
#   docker build --build-arg GIT_SHA=$(git rev-parse --short HEAD) -t companion:$(git rev-parse --short HEAD) .
#
# Needs BuildKit (default in Docker 23+). The final image holds only the server
# binary and the generated site (target/site: pkg/, fonts/ and anything else
# from public/). schema.surql is compiled into the binary (include_str! in
# src/server/db.rs), so it is not copied.
#
# Layer order, slowest-changing first:
#   1. base image + apt packages            (changes: almost never)
#   2. wasm32 target for the pinned rustc   (changes: when RUST_VERSION moves)
#   3. cargo-leptos prebuilt binary         (changes: when CARGO_LEPTOS_VERSION moves)
#   4. source copy + `cargo leptos build`   (changes: every commit)
# cargo-leptos downloads wasm-bindgen, Tailwind and wasm-opt from GitHub on a
# cold tool cache; one CI build failed on such a download, so the build step
# retries twice (a retry reuses everything already compiled).
# Step 4 uses BuildKit cache mounts for the cargo registry, the target dir and
# cargo-leptos' tool cache (wasm-bindgen, tailwind, wasm-opt), so a rebuild on
# the same host only recompiles this crate. Cache mounts are not exported with
# the image; see docs/RUNBOOK.md "Building the image" for CI caching.
#
# Measured in CI (M8): about 232 MB uncompressed: debian trixie-slim plus
# runtime packages, the stripped server binary (127 MB, embedded SurrealDB)
# and the site (release WASM 8.2 MB before compression, CSS 70 KB, JS, fonts).
# Cold build about 20 min on a 4 vCPU runner, about 9 min with warm mounts.

ARG RUST_VERSION=1.99.0
ARG DEBIAN_SUITE=trixie
ARG CARGO_LEPTOS_VERSION=0.3.11
# sha256 of cargo-leptos-x86_64-unknown-linux-gnu.tar.gz for the version above
# (from the release's .sha256 asset).
ARG CARGO_LEPTOS_SHA256=1c9cf0e1e3ffae6c8b58d29281d949dab560cea613b80d361f323f93071df7c8

# ---------------------------------------------------------------------------
# Builder
# ---------------------------------------------------------------------------
FROM rust:${RUST_VERSION}-slim-${DEBIAN_SUITE} AS builder
ARG RUST_VERSION
ARG CARGO_LEPTOS_VERSION
ARG CARGO_LEPTOS_SHA256

# rust-toolchain.toml says channel = "stable"; without this override rustup
# would download whatever "stable" is on the day of the build. Pinning the
# image tag and RUSTUP_TOOLCHAIN together makes the build reproducible.
# LEPTOS_TAILWIND_VERSION: the Tailwind build the dev machines use (the local
# cargo-leptos cache holds tailwindcss-v4.2.1).
ENV RUSTUP_TOOLCHAIN=${RUST_VERSION} \
    CARGO_TERM_COLOR=never \
    CARGO_INCREMENTAL=0 \
    LEPTOS_TAILWIND_VERSION=v4.2.1

# pkg-config + libssl-dev: openssl-sys (reqwest/riven native-tls).
# g++/make/cmake/perl: C/C++ build scripts (aws-lc-sys, ring, zstd-sys and co).
# git: only for `git init` below. binutils: strip.
RUN apt-get update \
 && apt-get install -y --no-install-recommends \
      ca-certificates curl git pkg-config libssl-dev g++ make cmake perl binutils \
 && rm -rf /var/lib/apt/lists/*

RUN rustup target add wasm32-unknown-unknown --toolchain "${RUST_VERSION}"

# cargo-leptos from the prebuilt release tarball (same as CI), checksum-verified.
RUN curl -sSfL -o /tmp/cargo-leptos.tar.gz \
      "https://github.com/leptos-rs/cargo-leptos/releases/download/v${CARGO_LEPTOS_VERSION}/cargo-leptos-x86_64-unknown-linux-gnu.tar.gz" \
 && echo "${CARGO_LEPTOS_SHA256}  /tmp/cargo-leptos.tar.gz" | sha256sum -c - \
 && tar -xzf /tmp/cargo-leptos.tar.gz -C /tmp \
 && install -m 0755 /tmp/cargo-leptos-x86_64-unknown-linux-gnu/cargo-leptos /usr/local/cargo/bin/cargo-leptos \
 && rm -rf /tmp/cargo-leptos* \
 && cargo leptos --version

WORKDIR /src
COPY . .

# Tailwind v4 finds class names by scanning the project and skips what
# .gitignore lists, but (like ripgrep) it may only honour .gitignore inside a
# git repository. .git is not in the build context, so create an empty repo:
# otherwise the scan walks target/ (a cache mount here).
RUN git init -q

RUN --mount=type=cache,id=companion-cargo-registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=companion-cargo-git,target=/usr/local/cargo/git \
    --mount=type=cache,id=companion-target,target=/src/target \
    --mount=type=cache,id=companion-leptos-tools,target=/root/.cache/cargo-leptos \
    for attempt in 1 2 3; do \
      cargo leptos build --release && break; \
      [ "$attempt" = 3 ] && exit 1; \
      echo "cargo leptos build failed (attempt $attempt); retrying in 20 s"; sleep 20; \
    done \
 && mkdir -p /out \
 && cp target/release/lol_team_companion /out/lol_team_companion \
 && strip /out/lol_team_companion \
 && cp -r target/site /out/site \
 && ls -la /out /out/site /out/site/pkg

# ---------------------------------------------------------------------------
# Runtime
# ---------------------------------------------------------------------------
FROM debian:${DEBIAN_SUITE}-slim AS runtime
ARG GIT_SHA=unknown

LABEL org.opencontainers.image.title="Companion" \
      org.opencontainers.image.description="Private coaching workspace for one team" \
      org.opencontainers.image.revision="${GIT_SHA}"

# libssl3t64: the binary links OpenSSL dynamically (native-tls); trixie renamed
# the package from libssl3 (bookworm). ca-certificates: outbound HTTPS to the
# Riot API and Data Dragon. tini: PID 1 that forwards SIGTERM and reaps.
# curl: HEALTHCHECK.
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates libssl3t64 tini curl \
 && rm -rf /var/lib/apt/lists/* \
 && groupadd --system --gid 10001 companion \
 && useradd --system --uid 10001 --gid companion --home-dir /app --no-create-home \
      --shell /usr/sbin/nologin companion \
 && mkdir -p /data/db /data/inbox /data/attachments \
 && chown -R companion:companion /data

WORKDIR /app
COPY --from=builder --chown=root:root /out/lol_team_companion /app/lol_team_companion
COPY --from=builder --chown=root:root /out/site /app/site

# The binary is not started by cargo-leptos, so Leptos reads its options from
# the environment (as in CI). LEPTOS_ENV=PROD: config.rs treats LEPTOS_ENV=DEV
# as a development run (no SESSION_SECRET requirement, insecure cookie).
# Never set APP_ENV in production.
ENV LEPTOS_OUTPUT_NAME=lol_team_companion \
    LEPTOS_SITE_ROOT=/app/site \
    LEPTOS_SITE_PKG_DIR=pkg \
    LEPTOS_SITE_ADDR=0.0.0.0:3020 \
    LEPTOS_RELOAD_PORT=3021 \
    LEPTOS_ENV=PROD \
    SURREAL_DATA_DIR=/data/db \
    BUG_REPORT_INBOX_PATH=/data/inbox/bug-reports.md \
    ATTACHMENTS_DIR=/data/attachments \
    RUST_LOG=info

# /data holds the SurrealKV directory (/data/db, exclusive lock: one process
# only), the bug-report inbox export (/data/inbox) and the feedback
# screenshots (/data/attachments, M141). A new named volume is
# initialised from this directory, so it starts owned by uid 10001.
VOLUME ["/data"]
USER 10001:10001
EXPOSE 3020
STOPSIGNAL SIGTERM

HEALTHCHECK --interval=30s --timeout=5s --start-period=90s --retries=3 \
  CMD curl -fsS --max-time 4 http://127.0.0.1:3020/healthz || exit 1

ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["/app/lol_team_companion"]
