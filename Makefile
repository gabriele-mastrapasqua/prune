.PHONY: build test install uninstall clean fmt lint run help

# Default target
all: build

# Build release binary (default)
build:
	cargo build --release

# Run all checks and tests
test: fmt lint
	cargo test

# Check formatting
fmt:
	cargo fmt --all -- --check

# Run clippy lints
lint:
	cargo clippy -- -D warnings

# Format code
fmt-fix:
	cargo fmt --all

# Build release binary and install to system
install: build
	@echo "Installing prune to /usr/local/bin..."
	@sudo cp target/release/prune /usr/local/bin/prune
	@sudo chmod +x /usr/local/bin/prune
	@echo "✓ prune installed successfully!"
	@echo "Run 'prune' to start the interactive TUI"

# Uninstall from system
uninstall:
	@echo "Removing prune from /usr/local/bin..."
	@sudo rm -f /usr/local/bin/prune
	@echo "✓ prune uninstalled"

# Clean build artifacts
clean:
	cargo clean

# Run the tool
run:
	cargo run --release

# Show help
help:
	@echo "Available targets:"
	@echo "  build        - Build release binary (default)"
	@echo "  test         - Run fmt, lint, and tests"
	@echo "  fmt          - Check formatting"
	@echo "  fmt-fix      - Fix formatting"
	@echo "  lint         - Run clippy"
	@echo "  install      - Build and install to /usr/local/bin"
	@echo "  uninstall    - Remove from /usr/local/bin"
	@echo "  clean        - Remove build artifacts"
	@echo "  run          - Run in release mode"
