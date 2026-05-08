# ── Stage 1: Build frontend ──────────────────────────────────────
FROM node:22-alpine AS frontend
WORKDIR /app/frontend
COPY frontend/package*.json ./
RUN npm ci
COPY frontend/ ./
RUN npm run build

# ── Stage 2: Build Rust backend ──────────────────────────────────
# rust-embed embeds ../frontend/build at compile time, so the frontend
# build must exist at /app/frontend/build before cargo build runs.
FROM rust:1-slim-bookworm AS builder
WORKDIR /app
COPY backend/ backend/
COPY --from=frontend /app/frontend/build frontend/build
WORKDIR /app/backend
RUN cargo build --release

# ── Stage 3: Minimal runtime ─────────────────────────────────────
FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/backend/target/release/clef-note .
COPY docker-entrypoint.sh .
RUN chmod +x docker-entrypoint.sh
EXPOSE 8080
ENTRYPOINT ["/app/docker-entrypoint.sh"]
