#!/bin/bash

# Build script for hetuflow-sdk WASM package
# Runs wasm-pack build and then post-processes the generated package.json

set -e

echo "Building WASM package with wasm-pack..."
# --target web or bundler
wasm-pack build --target web --out-dir examples/wasm/pkg --features with-wasm

echo "Running post-build script to add dependencies..."
node post-build.js

echo "WASM package build completed successfully!"
