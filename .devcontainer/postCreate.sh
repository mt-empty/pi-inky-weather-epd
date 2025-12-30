#!/usr/bin/env bash
set -euo pipefail

echo "[postCreate] Updating apt and installing tools..."
sudo apt-get update
sudo apt-get install -y --no-install-recommends \
  vim \
  nodejs
sudo rm -rf /var/lib/apt/lists/*

echo "[postCreate] Installing cargo-release (locked)..."
cargo install cargo-release --locked
cargo install cargo-insta --locked

echo "[postCreate] Installing latest difftastic (Linux x86_64)..."
latest_url="$(
  curl -fsSL https://api.github.com/repos/Wilfred/difftastic/releases/latest \
  | grep browser_download_url \
  | grep x86_64-unknown-linux-gnu.tar.gz \
  | cut -d '"' -f 4
)"

tmpdir="$(mktemp -d)"
curl -fsSL "$latest_url" -o "$tmpdir/difft.tar.gz"
tar -xzf "$tmpdir/difft.tar.gz" -C "$tmpdir"

sudo install -m 755 "$tmpdir"/difft /usr/local/bin/difft
rm -rf "$tmpdir"

echo "[postCreate] Done."
