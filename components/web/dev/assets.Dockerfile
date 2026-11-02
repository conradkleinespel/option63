FROM docker.io/library/node:22-bookworm-slim

RUN useradd --create-home --uid 1001 --shell /usr/sbin/nologin dev

ENV HOME=/home/dev

RUN mkdir -p /app/components/web \
    && chown -R dev:dev /home/dev /app

USER dev

WORKDIR /app/components/web

CMD ["sh", "-c", "if [ ! -f node_modules/.package-lock.json ]; then npm ci --no-audit --no-fund; fi; npm run build; npm run dev"]
