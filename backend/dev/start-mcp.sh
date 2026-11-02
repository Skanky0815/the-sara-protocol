#!/bin/sh
# Stable working directory for a desktop-launched stdio server.
set -eu
backend_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$backend_dir"
if [ ! -x target/release/the-sara-protocol ]; then
  echo 'MCP-Binary fehlt: zuerst cargo build --release --locked in backend/ ausführen.' >&2
  exit 1
fi
exec ./target/release/the-sara-protocol --mcp
