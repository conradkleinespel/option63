#!/usr/bin/env bash

set -xeuo pipefail

cargo test --verbose

if [ "$WITH_TARPAULIN" = 1 ]; then
  which cargo-tarpaulin || cargo install cargo-tarpaulin
  cargo tarpaulin
fi

which cargo-deny || cargo install cargo-deny
cargo deny check licenses