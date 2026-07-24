#!/bin/bash
# Build WASM module for browser usage
# Requires: wasm-pack (cargo install wasm-pack)

set -e

echo "Building dicelet-wasm..."

# Build for browser (web target)
wasm-pack build --target web --release --out-dir pkg

echo "Build complete! Output in crates/dicelet-wasm/pkg/"
echo ""
echo "Usage in browser:"
echo "  <script type=\"module\">"
echo "    import init, { roll, parse } from './pkg/dicelet_wasm.js';"
echo "    await init();"
echo "    const result = roll('4d6k3');"
echo "    console.log(result.full);"
echo "  </script>"