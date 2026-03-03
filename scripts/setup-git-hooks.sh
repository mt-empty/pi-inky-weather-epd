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

echo "Running cargo test with RUN_MODE=test..."

if ! RUN_MODE=test cargo test; then
    echo "❌ Tests failed. Please fix failing tests before pushing."
    exit 1
fi

echo "Running BOM snapshot tests..."

if ! RUN_MODE=test APP_API__PROVIDER=bom cargo test --test snapshot_provider_test; then
    echo "❌ BOM snapshot tests failed. Please fix failing tests before pushing."
    exit 1
fi

echo "Running Open-Meteo prefer_weather_codes snapshot tests..."

if ! RUN_MODE=test APP_RENDER_OPTIONS__PREFER_WEATHER_CODES=true cargo test --test snapshot_open_meteo_prefer_codes_test; then
    echo "❌ prefer_weather_codes snapshot tests failed. Please fix failing tests before pushing."
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

echo "✅ Pre-push hook installed successfully!"
echo ""
echo "The hook will run the following checks before each push:"
echo "  1. Code formatting (cargo fmt)"
echo "  2. Clippy linting (cargo clippy -- -D warnings)"
echo "  3. All tests (RUN_MODE=test cargo test)"
echo "  4. BOM snapshot tests"
echo "  5. Open-Meteo prefer_weather_codes snapshot tests"
echo "  6. Version tag validation"
echo ""
echo "🎉 Git hooks setup complete!"
