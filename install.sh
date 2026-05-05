#!/bin/sh
# Install E globally from Rust source

set -e

cd "$(dirname "$0")/e"
echo "🔨 Building E..."
cargo build --release
echo "📦 Installing to /usr/local/bin/e..."
sudo cp target/release/e /usr/local/bin/e
echo "✅ Done. Type 'e' to use."
