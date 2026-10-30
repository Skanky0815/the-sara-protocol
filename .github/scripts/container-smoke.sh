#!/usr/bin/env bash
set -euo pipefail
export BACKEND_IMAGE="${BACKEND_IMAGE:?}"
export POSTGRES_PASSWORD=sara_isolated_ci_test
export BACKEND_PORT=53043
project="sara-image-${GITHUB_RUN_ID:-local}"
compose=(docker compose -p "$project" -f compose.acceptance.yaml)
cleanup() { "${compose[@]}" down --volumes; }
trap cleanup EXIT
"${compose[@]}" up --no-build --pull never -d --wait --wait-timeout 120
curl --fail --silent "http://127.0.0.1:$BACKEND_PORT/health" | python3 -c 'import json,sys; assert json.load(sys.stdin)=={"status":"ok"}'
"${compose[@]}" restart backend
"${compose[@]}" up --no-build --pull never -d --wait --wait-timeout 120
curl --fail --silent "http://127.0.0.1:$BACKEND_PORT/openapi.json" | python3 -c 'import json,sys; assert "paths" in json.load(sys.stdin)'
