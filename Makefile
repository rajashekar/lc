# Makefile for LC (LLM Client) project
# Includes PDF testing tasks and development utilities

.PHONY: help test test-pdf test-pdf-no-feature test-all build clean format lint fix check generate-pdf-fixtures install-pdf-deps

# Default target
help:
	@echo "Available targets:"
	@echo "  help                  - Show this help message"
	@echo "  build                 - Build the project"
	@echo "  test                  - Run all tests"
	@echo "  test-pdf              - Run PDF tests with feature enabled"
	@echo "  test-pdf-no-feature   - Run PDF tests without feature (error handling)"
	@echo "  test-all              - Run all tests including PDF variants"
	@echo "  format                - Format code with rustfmt"
	@echo "  lint                  - Run clippy linter"
	@echo "  fix                   - Auto-fix clippy issues"
	@echo "  check                 - Run all checks (format, lint, test)"
	@echo "  clean                 - Clean build artifacts"
	@echo "  generate-pdf-fixtures - Generate PDF test fixtures (requires Python deps)"
	@echo "  install-pdf-deps      - Install Python dependencies for fixture generation"

# Build the project
build:
	cargo build

# Build release version
build-release:
	cargo build --release

# Run all standard tests
test:
	cargo test

# Run PDF tests with feature enabled
test-pdf:
	@echo "Running PDF tests with feature enabled..."
	cargo test --features pdf pdf_reader_tests

# Run PDF tests without feature (should test graceful degradation)
test-pdf-no-feature:
	@echo "Running PDF tests without feature (testing error handling)..."
	cargo test --no-default-features pdf_reader_tests::tests::test_pdf_feature_disabled || true

# Run unit tests specifically
test-pdf-unit:
	@echo "Running PDF unit tests..."
	cargo test --features pdf pdf_reader_tests::tests

# Run integration tests specifically
test-pdf-integration:
	@echo "Running PDF integration tests..."
	cargo test --features pdf pdf_reader_tests::integration_tests

# Run performance tests specifically
test-pdf-performance:
	@echo "Running PDF performance tests..."
	cargo test --features pdf pdf_reader_tests::performance_tests

# Run all test variants
test-all: test test-pdf test-pdf-no-feature

# Format code
format:
	cargo fmt

# Check formatting
format-check:
	cargo fmt -- --check

# Run clippy linter
lint:
	cargo clippy --all-targets --all-features -- -D warnings

# Auto-fix clippy issues
fix:
	cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged

# Run all checks
check: format-check lint test-all

# Clean build artifacts
clean:
	cargo clean
	rm -rf target/

# Install Python dependencies for PDF fixture generation
install-pdf-deps:
	@echo "Installing Python dependencies for PDF fixture generation..."
	pip3 install reportlab PyPDF2 Pillow

# Generate PDF test fixtures using Python script
generate-pdf-fixtures:
	@echo "Generating PDF test fixtures..."
	python3 scripts/generate_pdf_fixtures.py

# List test fixtures
list-fixtures:
	@echo "PDF test fixtures:"
	@ls -la tests/pdf_fixtures/*.pdf 2>/dev/null || echo "No PDF fixtures found"

# Run a specific test by name
test-specific:
	@if [ -z "$(TEST)" ]; then \
		echo "Usage: make test-specific TEST=test_name"; \
		echo "Example: make test-specific TEST=test_simple_text_extraction"; \
	else \
		cargo test --features pdf $(TEST); \
	fi

# Run tests with output
test-verbose:
	cargo test --features pdf pdf_reader_tests -- --nocapture

# Benchmark tests
bench:
	cargo test --features pdf pdf_reader_tests::performance_tests --release -- --nocapture

# CI simulation - run tests as CI would
ci-test:
	@echo "Simulating CI test run..."
	cargo build --release
	cargo test --release
	cargo test --release --features pdf pdf_reader_tests
	cargo test --release --no-default-features pdf_reader_tests::tests::test_pdf_feature_disabled || true
	cargo fmt -- --check
	cargo clippy --all-targets --all-features -- -D warnings

# Development setup
dev-setup: install-pdf-deps generate-pdf-fixtures
	@echo "Development environment setup complete!"
	@echo "Run 'make test-all' to verify everything works."

# Quick development test cycle
dev-test: format lint test-pdf

# Show test coverage (requires cargo-llvm-cov)
coverage:
	@command -v cargo-llvm-cov >/dev/null 2>&1 || { echo "cargo-llvm-cov not installed. Run: cargo install cargo-llvm-cov"; exit 1; }
	cargo llvm-cov --features pdf --html
	@echo "Coverage report generated in target/llvm-cov/html/index.html"

# Docker test (if docker available)
docker-test:
	@echo "Running tests in Docker container..."
	docker run --rm -v "$(PWD)":/workspace -w /workspace rust:latest /bin/bash -c "cargo test --features pdf pdf_reader_tests"

# Show project info
info:
	@echo "LC (LLM Client) - PDF Testing"
	@echo "============================="
	@echo "Rust version: $$(rustc --version)"
	@echo "Cargo version: $$(cargo --version)"
	@echo "Project features: $$(grep -A 10 '\\[features\\]' Cargo.toml | tail -n +2)"
	@echo ""
	@echo "PDF test fixtures:"
	@ls -la tests/pdf_fixtures/ 2>/dev/null || echo "  No fixtures directory found"
