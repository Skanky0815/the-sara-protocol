#!/usr/bin/env bash
# Print an existing digest; only a definite missing manifest is treated as absent.
set -euo pipefail
ref="${1:?Image-Referenz fehlt}"
error_file=$(mktemp)
trap 'rm -f "$error_file"' EXIT
if digest=$(docker buildx imagetools inspect "$ref" --format '{{.Manifest.Digest}}' 2>"$error_file"); then
  [[ "$digest" =~ ^sha256:[a-f0-9]{64}$ ]]
  printf '%s\n' "$digest"
elif grep -Eqi 'manifest unknown|not found|not_found' "$error_file"; then
  printf '\n'
else
  cat "$error_file" >&2
  exit 1
fi
