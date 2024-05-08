#!/bin/bash
set -euo pipefail
mkdir -p generated

RPC_URL="${RPC_URL:-"http://localhost:9933"}"

curl \
  "$RPC_URL" \
  -d '{"jsonrpc":"2.0","method":"state_getMetadata","id":1}' \
  -H 'Content-Type: application/json' |
  jq -r '.result' |
  xxd -r -p >./generated/humanode_metadata.scale
