#!/bin/sh
set -eu

umask 077

cat > /config/web.json <<'JSON'
{
  "web": {
    "listen": "0.0.0.0:8080",
    "database": {
      "host": "postgres",
      "port": 5432,
      "name": "option63",
      "username": "option63",
      "password": "option63",
      "tls": {
        "enabled": false
      }
    }
  }
}
JSON

chmod 600 /config/web.json
