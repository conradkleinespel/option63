FROM docker.io/library/rust:1-bookworm

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        pkg-config libssl-dev ca-certificates curl xz-utils \
    && rm -rf /var/lib/apt/lists/*

RUN set -eux; \
    case "$(dpkg --print-architecture)" in \
      amd64) target="x86_64-unknown-linux-musl"; \
             checksum="35ea667cbe4bc96c23ba2a1f46c94c46bc385b67165e3bf621c0e7e4959aa99b"; : exclude-from-detect-secrets ;; \
      arm64) target="aarch64-unknown-linux-musl"; \
             checksum="3961878b21e3c1729279322f4007dd5bbf0032110fbcb5e7b983946a9ceb3bf6"; : exclude-from-detect-secrets ;; \
      *) echo "unsupported architecture: $(dpkg --print-architecture)" >&2; exit 1 ;; \
    esac; \
    archive="watchexec-2.8.0-${target}.tar.xz"; \
    curl --fail --location --silent --show-error \
      --output "/tmp/${archive}" \
      "https://github.com/watchexec/watchexec/releases/download/v2.8.0/${archive}"; \
    echo "${checksum}  /tmp/${archive}" | sha256sum --check --strict; \
    tar -xJf "/tmp/${archive}" -C /usr/local/bin \
      --strip-components=1 "watchexec-2.8.0-${target}/watchexec"; \
    rm "/tmp/${archive}"; \
    watchexec --version

RUN useradd --create-home --uid 1001 --shell /usr/sbin/nologin dev

ENV HOME=/home/dev
ENV CARGO_HOME=/home/dev/.cargo
ENV RUSTUP_HOME=/usr/local/rustup

RUN mkdir -p /home/dev/.cargo /app \
    && chown -R dev:dev /home/dev /app

USER dev

WORKDIR /app
