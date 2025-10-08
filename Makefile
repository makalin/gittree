.PHONY: build-go build-rs build-all clean test lint install-go install-rs

# Build Go backend
build-go:
	@echo "Building Go backend..."
	@go build -o bin/gittree-go ./cmd/gittree

# Build Rust backend  
build-rs:
	@echo "Building Rust backend..."
	@cargo build --release
	@cp target/release/gittree bin/gittree-rs

# Build both backends
build-all: build-go build-rs

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@rm -rf bin/
	@cargo clean

# Run tests
test:
	@echo "Running Go tests..."
	@go test ./...
	@echo "Running Rust tests..."
	@cargo test

# Run linters
lint:
	@echo "Running Go linters..."
	@golangci-lint run
	@echo "Running Rust linters..."
	@cargo clippy -- -D warnings
	@cargo fmt -- --check

# Install Go version
install-go: build-go
	@echo "Installing Go version..."
	@cp bin/gittree-go /usr/local/bin/gittree

# Install Rust version
install-rs: build-rs
	@echo "Installing Rust version..."
	@cp bin/gittree-rs /usr/local/bin/gittree

# Default target
all: build-all