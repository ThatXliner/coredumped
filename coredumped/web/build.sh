#!/bin/bash
set -e

cd "$(dirname "$0")/.."

echo "Building WASM..."
wasm-pack build --target web --out-dir web/pkg --no-default-features --features web,prelude

echo "Done! Serve the web/ directory with a local server:"
echo "  cd web && python3 -m http.server 8080"
echo "  Then open http://localhost:8080"
