# Makefile for convenient building
# Usage: make [target]

# Git Commit Rewriter - Makefile
# Use `make help` to list available commands

.PHONY: help build clean test install macos-app windows-portable linux

# Variables
PROJECT = commit-rewriter
VERSION = 2.0.0
BUILD_DIR = target/release
DIST_DIR = releases

# Colors for output
GREEN = \033[0;32m
YELLOW = \033[1;33m
NC = \033[0m # No Color

help: ## Show help
	@echo "Available commands:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(GREEN)%-20s$(NC) %s\n", $$1, $$2}'

build: ## Build for current platform
	@echo "$(GREEN)Building for current platform...$(NC)"
	cargo build --release
	@ls -lh $(BUILD_DIR)/$(PROJECT)

clean: ## Clean build artifacts
	@echo "$(YELLOW)Cleaning...$(NC)"
	cargo clean
	rm -rf $(DIST_DIR)

test: ## Run tests
	cargo test

install: build ## Install to system (requires sudo)
	@echo "$(GREEN)Installing...$(NC)"
	sudo cp $(BUILD_DIR)/$(PROJECT) /usr/local/bin/$(PROJECT)
	@echo "$(GREEN)✅ Installed to /usr/local/bin/$(PROJECT)$(NC)"

# Platform-specific builds
macos-app-arm64: ## Create .app and .dmg for macOS (Apple Silicon)
	@./scripts/build-macos-app.sh arm64

macos-app-x86_64: ## Create .app and .dmg for macOS (Intel)
	@./scripts/build-macos-app.sh x86_64

macos-app: ## Create .app and .dmg for macOS (auto-detect architecture)
	@./scripts/build-macos-app.sh

windows-portable: ## Create portable Windows .exe without console (cross-compilation)
	@./scripts/build-windows.sh

linux-x86_64: ## Build for Linux (x86_64)
	@./scripts/build-linux.sh x86_64

linux-arm64: ## Build for Linux (ARM64)
	@./scripts/build-linux.sh arm64

linux: linux-x86_64 ## Build for Linux (default: x86_64)

# Dependency check
check-deps: ## Check system dependencies
	@echo "$(GREEN)Checking dependencies...$(NC)"
	@command -v rustc >/dev/null 2>&1 || { echo "$(YELLOW)⚠️  Rust is not installed$(NC)"; exit 1; }
	@command -v cargo >/dev/null 2>&1 || { echo "$(YELLOW)⚠️  Cargo is not installed$(NC)"; exit 1; }
	@command -v git >/dev/null 2>&1 || { echo "$(YELLOW)⚠️  Git is not installed$(NC)"; exit 1; }
	@echo "$(GREEN)✅ All dependencies installed$(NC)"

# Binary size
size: build ## Show binary size
	@echo "$(GREEN)Binary size:$(NC)"
	@ls -lh $(BUILD_DIR)/$(PROJECT) | awk '{print $$5}'

# Dependency information
deps-info: ## Show dependency information
	@echo "$(GREEN)Dependencies:$(NC)"
	@cargo tree --depth 1

# Update dependencies
update: ## Update dependencies
	cargo update

# Code formatting
fmt: ## Format code
	cargo fmt

# Linter check
clippy: ## Run clippy
	cargo clippy -- -D warnings

# Full check before commit
check: fmt clippy test ## Formatting, linter and tests
