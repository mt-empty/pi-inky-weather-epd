#!/bin/bash

# Setup script for installing Git hooks
# Run this script when setting up the development environment

set -e

echo "🔧 Setting up Git hooks for pi-inky-weather-epd..."

# Ensure we're in the git repository root
if [ ! -d ".git" ]; then
    echo "❌ Error: This script must be run from the repository root"
    exit 1
fi

# Create hooks directory if it doesn't exist
mkdir -p .git/hooks

# Install pre-commit hook
echo "📝 Installing pre-commit hook..."
cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash

staged_rs_files=$(git diff --cached --name-only --diff-filter=ACM | grep '\.rs$')

if [ -z "$staged_rs_files" ]; then
    exit 0
fi

echo "Running cargo fmt on staged Rust files..."

while IFS= read -r file; do
    if [ -f "$file" ]; then
        rustfmt "$file"
    fi
done <<< "$staged_rs_files"

while IFS= read -r file; do
    if [ -f "$file" ]; then
        git add "$file"
    fi
done <<< "$staged_rs_files"

echo "✅ cargo fmt applied to staged files."
EOF

chmod +x .git/hooks/pre-commit

# Install pre-push hook
echo "📝 Installing pre-push hook..."
cat > .git/hooks/pre-push << 'EOF'
#!/bin/bash

echo "Running pre-push hook..."

# Run cargo fmt
echo "Running cargo fmt..."
if ! cargo fmt -- --check; then
    echo "❌ Formatting check failed. Please run 'cargo fmt' to fix formatting issues."
    exit 1
fi

# Run cargo clippy
echo "Running cargo clippy..."
if ! cargo clippy -- -D warnings; then
    echo "❌ Clippy check failed. Please fix clippy warnings before pushing."
    exit 1
fi

# A single run covers all providers and render options: each test builds
# its own settings value (see tests/helpers).
echo "Running cargo test..."

if ! cargo test; then
    echo "❌ Tests failed. Please fix failing tests before pushing."
    exit 1
fi

# Checking if latest tag matches that in cargo.toml
echo "Checking if latest tag matches that in Cargo.toml..."

LATEST_TAG=$(git describe --tags "$(git rev-list --tags --max-count=1)")
if [ -z "$LATEST_TAG" ]; then
    echo "❌ No tags found. Please create a tag before pushing."
    exit 1
fi

CARGO_VERSION="v$(grep --max-count 1 -oP 'version\s*=\s*"\K.+(?=")' Cargo.toml)"
if [ "$LATEST_TAG" != "$CARGO_VERSION" ]; then
    echo "❌ Latest tag ($LATEST_TAG) does not match version in Cargo.toml ($CARGO_VERSION). Please update the tag or Cargo.toml."
    exit 1
fi

echo "✅ Pre-push checks passed!"
EOF

# Make the hook executable
chmod +x .git/hooks/pre-push

echo "✅ Git hooks installed successfully!"
echo ""
echo "pre-commit: auto-formats staged .rs files with rustfmt"
echo ""
echo "pre-push:"
echo "  1. Code formatting (cargo fmt)"
echo "  2. Clippy linting (cargo clippy -- -D warnings)"
echo "  3. All tests (cargo test — covers all providers and render options)"
echo "  4. Version tag validation"
echo ""
echo "🎉 Git hooks setup complete!"
