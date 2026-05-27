#!/bin/bash
set -e

cd "$(dirname "$0")/.."

echo "Building WASM..."
wasm-pack build --target web --out-dir web-assets/pkg

echo "Done! Serve the web-assets/ directory with a local server:"
echo "  cd web-assets && python3 -m http.server 8080"
echo "  Then open http://localhost:8080"
