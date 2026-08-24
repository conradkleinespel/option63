# syntax=docker/dockerfile:1

# ---- Stage 1: build frontend assets ----
FROM docker.io/node:22-slim AS frontend
WORKDIR /app/web
COPY web/package.json web/package-lock.json* ./
RUN npm ci --no-audit --no-fund
COPY web/ ./
RUN npm run build

# ---- Stage 2: build Rust binary ----
FROM docker.io/rust:1-bookworm AS backend
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
COPY vcard-derives/ vcard-derives/
COPY vcard-lib/ vcard-lib/
COPY vcard-bin/ vcard-bin/
COPY web/ web/
COPY --from=frontend /app/web/static web/static
RUN cargo build --release --bin option63-web

# ---- Stage 3: runtime ----
FROM docker.io/debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
RUN useradd --no-create-home --no-log-init --system --uid 1001 option63
COPY --from=backend /app/target/release/option63-web /app/option63-web
COPY web/templates /app/web/templates
COPY --from=frontend /app/web/static /app/web/static
RUN chown -R option63:option63 /app
USER option63

ENV PORT=8080
EXPOSE 8080
CMD ["/app/option63-web"]
