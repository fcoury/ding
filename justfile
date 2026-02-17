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
    @echo "Done. Run 'ding send \"hello\"' to test."

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
