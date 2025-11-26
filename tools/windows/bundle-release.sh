#!/bin/bash
set -e # Exit immediately if a command fails

echo "=== Setting up Build Environment ==="

# 1. Compile the Rustc Wrapper
echo "Compiling rustc-wrapper..."
rustc tools/windows/rustc-wrapper/rustc-wrapper.rs -o target/rustc-wrapper.exe

# 2. Configure Environment Variables
# Use cygpath to ensure Windows-compatible path for Cargo
export RUSTC_WRAPPER=$(cygpath -w $(pwd)/target/rustc-wrapper.exe)

# Limit concurrency to prevent OOM crashes on Triton-VM
export CARGO_BUILD_JOBS=1

echo "Wrapper set to: $RUSTC_WRAPPER"
echo "Concurrency set to: $CARGO_BUILD_JOBS"

# 3. Clean triton-vm artifacts to ensure the wrapper is used
# (Optional for CI which starts clean, but good for local dev)
echo "Cleaning triton-vm artifacts to force stack fix..."
find target -type f -name "build-script-build*.exe" -exec rm -rf {} + 2>/dev/null || true

# 4. Run the Bundler
echo "=== Starting dx bundle ==="
cd desktop
dx bundle --release

echo "=== Build Complete ==="
