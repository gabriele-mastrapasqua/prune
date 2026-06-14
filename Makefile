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

# Build release binary and install to user bin directory
install: build
	@mkdir -p ~/.local/bin
	@cp target/release/prune ~/.local/bin/prune
	@chmod +x ~/.local/bin/prune
	@echo "✓ prune installed to ~/.local/bin/prune"
	@echo "Run 'prune' to start the interactive TUI"
	@if ! echo $$PATH | grep -q "$$HOME/.local/bin"; then \
		echo ""; \
		echo "Note: ~/.local/bin is not in your PATH."; \
		echo "Add this to your ~/.zshrc or ~/.bashrc:"; \
		echo '  export PATH="$$HOME/.local/bin:$$PATH"'; \
	fi

# Uninstall from user bin directory
uninstall:
	@echo "Removing prune from ~/.local/bin..."
	@rm -f ~/.local/bin/prune
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
