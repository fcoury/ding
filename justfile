# Build and install ding, refreshing all app bundles

# Build in release mode
build:
    cargo build --release

# Install ding to ~/.cargo/bin and refresh app bundles
install: build
    cargo install --path .
    @echo "Clearing cached app bundles so they get rebuilt with the new binary..."
    rm -rf ~/.cache/ding/apps
    @echo "Sending a test notification to rebuild the default bundle..."
    ding send --silent "ding installed successfully"
    @just _symlink-apps
    @echo "Done. Run 'ding send \"hello\"' to test."

# Symlink app bundles into ~/Applications so they appear in Focus settings
_symlink-apps:
    @mkdir -p ~/Applications
    @ln -sf ~/.cache/ding/apps/default.app ~/Applications/Ding.app
    @[ -d ~/.cache/ding/apps/claude.app ] && ln -sf ~/.cache/ding/apps/claude.app ~/Applications/"Ding Claude.app" || true
    @[ -d ~/.cache/ding/apps/codex.app ] && ln -sf ~/.cache/ding/apps/codex.app ~/Applications/"Ding Codex.app" || true

# Quick dev build + test notification
dev:
    cargo run -- send "dev build test"

# Run tests
test:
    cargo test

# Clean build artifacts and cached bundles
clean:
    cargo clean
    rm -rf ~/.cache/ding/apps
