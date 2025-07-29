# Testing Guide for LC (LLM Client)

This document explains the testing patterns used in the LC project and how to run the comprehensive test suite for all CLI commands.

## Rust Testing Patterns Used

### 1. Unit Tests (src/cli/tests.rs)
Unit tests are placed in the same module as the code they test, using the `#[cfg(test)]` attribute. These test internal functionality and are good for testing individual functions and methods.

### 2. Integration Tests (tests/ directory)
Integration tests are placed in the `tests/` directory at the project root. Each file in this directory is compiled as a separate crate and tests the public API. This is the **preferred approach for CLI testing** and is similar to testing patterns in Java/Python.

### 3. Test Organization Structure

**Integration Tests (Recommended for CLI commands):**
```
tests/
├── common/
│   └── mod.rs              # Shared test utilities and helpers
├── provider_commands.rs    # Provider command tests
├── key_commands.rs         # API key management tests
├── config_commands.rs      # Configuration management tests
└── ...                     # Additional command test files
```

**Unit Tests (For internal functionality):**
```
src/
├── cli/
│   └── tests.rs           # Unit tests for CLI internals
└── ...
```

## Test Coverage for All Commands

The test suite covers all CLI commands with both unit and integration tests:

### Provider Commands (`lc providers`) - 21 Integration Tests
- ✅ **Add**: Basic addition, custom paths, multiple providers
- ✅ **Update**: URL changes, provider preservation
- ✅ **Remove**: Existing/non-existent providers, bulk removal
- ✅ **List**: Empty lists, multiple providers, sorting
- ✅ **API Keys**: Setting, updating, removing keys
- ✅ **Headers**: Adding, removing, listing, isolation per provider
- ✅ **Token URLs**: Setting URLs, cached token management

### Key Management Commands (`lc keys`) - 12 Integration Tests
- ✅ **Add**: Adding keys for existing/non-existent providers
- ✅ **List**: Empty lists, mixed provider states
- ✅ **Get**: Retrieving existing keys, handling missing keys
- ✅ **Remove**: Removing keys, idempotent operations

### Configuration Commands (`lc config`) - 11 Integration Tests
- ✅ **Set**: Default provider, model, system prompt, max tokens, temperature
- ✅ **Get**: Retrieving existing/unset configuration values
- ✅ **Delete**: Removing configuration values
- ✅ **Validation**: Max tokens parsing, temperature parsing, template resolution

### Legacy Unit Tests (33 tests in src/cli/tests.rs)
- ✅ All original provider functionality tests
- ✅ Command structure validation
- ✅ Edge cases and error handling
- ✅ Integration workflows

## Test Helper Functions

### `create_test_config()`
Creates a temporary configuration with a temp directory for isolated testing.

### `create_test_provider_config(endpoint)`
Creates a standardized test provider configuration.

### `create_config_with_providers()`
Creates a configuration pre-populated with test providers (OpenAI and Anthropic).

## Running Tests

### Run All Tests
```bash
# Run all tests (unit + integration)
cargo test

# Run only integration tests
cargo test --tests

# Run only unit tests
cargo test --lib
```

### Run Specific Integration Test Files
```bash
# Provider command tests
cargo test --test provider_commands

# Key management tests
cargo test --test key_commands

# Configuration tests
cargo test --test config_commands
```

### Run Specific Test Modules
```bash
# Provider add functionality
cargo test provider_add_tests

# API key management
cargo test key_add_tests

# Configuration validation
cargo test config_validation_tests
```

### Run Individual Tests
```bash
# Test basic provider addition
cargo test test_provider_add_basic

# Test key management
cargo test test_key_add_for_existing_provider

# Test config parsing
cargo test test_max_tokens_parsing
```

### Run Tests with Output
```bash
cargo test -- --nocapture
```

### Run Tests in Verbose Mode
```bash
cargo test -- --test-threads=1 --nocapture
```

## Test Results Summary

As of the latest run, all **77 tests** are passing:

```
Integration Tests:
- Provider Commands: 21 passed
- Key Commands: 12 passed
- Config Commands: 11 passed

Unit Tests:
- CLI Module: 33 passed

Total: 77 passed; 0 failed; 0 ignored
```

## Test Categories Breakdown

1. **Basic Functionality Tests (13 tests)**
   - Provider CRUD operations
   - API key management
   - Header management
   - Token URL management
   - Cached token management

2. **Command Structure Tests (6 tests)**
   - Validation of command parsing
   - Parameter validation
   - Command aliases

3. **Edge Case Tests (6 tests)**
   - Input validation
   - Error handling
   - Special characters
   - Case sensitivity

4. **Integration Tests (8 tests)**
   - Complete workflows
   - Multi-provider scenarios
   - Error scenarios
   - Concurrent operations
   - Alias workflows
   - Configuration persistence

## Key Testing Principles

### 1. Isolation
Each test is isolated and doesn't depend on external state or other tests.

### 2. Comprehensive Coverage
Tests cover:
- Happy path scenarios
- Error conditions
- Edge cases
- Integration scenarios

### 3. Realistic Data
Tests use realistic provider names, URLs, and configurations.

### 4. Clear Assertions
Each test has clear, specific assertions that validate expected behavior.

### 5. Documentation
Tests serve as documentation for how the provider commands should behave.

## Adding New Tests

### For New CLI Commands

1. **Create a new integration test file** in `tests/` directory:
   ```rust
   // tests/new_command.rs
   mod common;
   
   use common::{create_config_with_providers, assertions};
   use lc::config::Config;
   
   #[cfg(test)]
   mod new_command_tests {
       use super::*;
       
       #[test]
       fn test_new_command_basic() {
           // Test implementation
       }
   }
   ```

2. **Add test modules for each subcommand**:
   - `new_command_add_tests`
   - `new_command_list_tests`
   - `new_command_remove_tests`

3. **Use shared utilities** from `tests/common/mod.rs`

### For New Provider Functionality

Add tests to the appropriate existing file:

```rust
#[test]
fn test_new_provider_feature() {
    // Arrange
    let mut config = create_config_with_providers();
    
    // Act
    let result = config.new_feature("test-provider".to_string());
    
    // Assert
    assert!(result.is_ok());
    assertions::assert_provider_exists(&config, "test-provider");
}
```

### Test File Naming Convention

- `tests/command_name.rs` - Integration tests for CLI commands
- `src/module/tests.rs` - Unit tests for internal functionality
- `tests/common/mod.rs` - Shared test utilities

### Benefits of This Structure

1. **Scalability**: Easy to add new command test files
2. **Isolation**: Each command has its own test file
3. **Reusability**: Shared utilities in `common/` module
4. **Maintainability**: Clear separation of concerns
5. **Familiar Pattern**: Similar to Java/Python testing structures

## Dependencies

The test suite uses these dependencies:
- `tempfile` - For creating temporary directories and files
- `chrono` - For date/time handling in cached tokens
- `toml` - For configuration serialization testing

These are included in the `[dev-dependencies]` section of `Cargo.toml`.