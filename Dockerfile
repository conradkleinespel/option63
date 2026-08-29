# syntax=docker/dockerfile:1

# ---- Stage 1: build frontend assets ----
FROM docker.io/node:22-slim AS frontend
WORKDIR /app/web
COPY components/web/package.json components/web/package-lock.json* ./
RUN npm ci --no-audit --no-fund
COPY components/web/ ./
RUN npm run build

# ---- Stage 2: build Rust binary ----
FROM docker.io/rust:1-bookworm AS backend
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
COPY components/derive/ components/derive/
COPY components/lib/ components/lib/
COPY components/cli/ components/cli/
COPY components/web/ components/web/
COPY --from=frontend /app/web/static components/web/static
RUN cargo build --release --bin o63web

# ---- Stage 3: runtime ----
FROM docker.io/debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
RUN useradd --no-create-home --no-log-init --system --uid 1001 option63
COPY --from=backend /app/target/release/o63web /app/o63web
COPY components/web/templates /app/web/templates
COPY components/web/src/assets /app/web/src/assets
COPY --from=frontend /app/web/static components/web/static
RUN chown -R option63:option63 /app
USER option63

ENV PORT=8080
EXPOSE 8080
CMD ["/app/o63web"]
