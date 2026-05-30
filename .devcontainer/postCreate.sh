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

echo "[postCreate] Installing latest difftastic..."
arch="$(uname -m)"  # x86_64 or aarch64 — detect at runtime for arm64/Apple Silicon support
latest_url="$(
  curl -fsSL https://api.github.com/repos/Wilfred/difftastic/releases/latest \
  | grep browser_download_url \
  | grep "${arch}-unknown-linux-gnu.tar.gz" \
  | cut -d '"' -f 4
)"

tmpdir="$(mktemp -d)"
curl -fsSL "$latest_url" -o "$tmpdir/difft.tar.gz"
tar -xzf "$tmpdir/difft.tar.gz" -C "$tmpdir"

sudo install -m 755 "$tmpdir"/difft /usr/local/bin/difft
rm -rf "$tmpdir"

echo "[postCreate] Installing gh copilot extension..."
# Must run as the container user (not root) so the extension lands in the correct
# user home and is accessible when running 'gh copilot suggest'.
if command -v gh &>/dev/null; then
    if ! gh extension install github/gh-copilot --force; then
        echo "[postCreate] WARNING: gh copilot extension install failed (likely unauthenticated)."
        echo "  Run: gh auth login && gh extension install github/gh-copilot"
    fi
else
    echo "[postCreate] WARNING: gh not found — skipping gh copilot extension install."
fi

echo "[postCreate] Done."
