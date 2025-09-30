#!/bin/bash
# must be run in the hetuflow/hetuflow-sdk directory of the project
# Build WASM example for hetuflow-sdk

set -e

echo "Building Hetuflow SDK WASM package..."
# Build the WASM package
echo "Running wasm-pack..."
wasm-pack build --target web --out-dir examples/wasm/pkg --features wasm

echo "Build completed successfully!"
echo ""
echo "To run the example:"
echo "  cd examples/wasm"
echo "  python3 -m http.server 8080"
echo ""
echo "Then open http://localhost:8080 in your browser"
