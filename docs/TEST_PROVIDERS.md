# Test Provider Management System

This document describes the test provider management system that prevents tests from overriding real configuration files in `~/Library/Application Support/lc/providers`.

## Overview

The test system now uses a "test-" prefix for all provider configurations created during testing. This ensures that:

1. Tests don't interfere with real provider configurations
2. Test providers are automatically cleaned up after test runs
3. Multiple test runs don't leave behind stale configuration files
4. Real user configurations are preserved

## Implementation Details

### Test Provider Prefix

All test providers use the prefix `test-` in their names:
- `openai` becomes `test-openai`
- `anthropic` becomes `test-anthropic`
- Custom providers follow the same pattern: `test-{provider_name}`

### Automatic Cleanup

The system provides automatic cleanup through several mechanisms:

1. **TestGuard RAII**: Automatically cleans up when tests complete
2. **Manual cleanup functions**: Can be called explicitly
3. **Initialization cleanup**: Removes stale test providers from previous runs

### File Structure

```
src/
├── cli/
│   └── tests.rs          # Updated test configuration with prefixed providers
├── test_utils.rs         # Test utilities and cleanup functions
└── lib.rs               # Module declarations

tests/
├── common/
│   └── mod.rs           # Updated common test utilities with prefixed providers
├── sync_commands.rs      # Cloud provider tests (not affected)
├── provider_commands.rs  # Updated to use test utilities and prefixed names
└── ...                  # Other test files
```

## Usage in Tests

### Using the Test Macro (Recommended)

```rust
use lc::test_with_cleanup;

#[test]
fn my_test() {
    test_with_cleanup!({
        // Your test code here
        // Cleanup happens automatically
    });
}
```

### Using TestGuard Directly

```rust
use lc::test_utils::TestGuard;

#[test]
fn my_test() {
    let _guard = TestGuard::new();
    // Your test code here
    // Cleanup happens when _guard is dropped
}
```

### Manual Setup/Teardown

```rust
use lc::cli::tests::{setup_tests, teardown_tests};

#[test]
fn my_test() {
    setup_tests();
    // Your test code here
    teardown_tests();
}
```

## Key Functions

### Test Provider Creation

```rust
// Get prefixed provider name
let provider_name = get_test_provider_name("openai"); // Returns "test-openai"

// Create config with test providers
let config = create_config_with_providers(); // Uses prefixed names
```

### Cleanup Functions

```rust
// Clean up test providers
cleanup_test_providers()?;

// Setup tests (includes cleanup)
setup_tests();

// Teardown tests (cleanup)
teardown_tests();
```

## Configuration Changes

### Before (Problematic)

```rust
config.providers.insert(
    "openai".to_string(),
    create_test_provider_config("https://api.openai.com"),
);
```

This would create/override `~/Library/Application Support/lc/providers/openai.json`

### After (Safe)

```rust
config.providers.insert(
    get_test_provider_name("openai"), // "test-openai"
    create_test_provider_config("https://api.openai.com"),
);
```

This creates `~/Library/Application Support/lc/providers/test-openai.json` which is automatically cleaned up.

## Benefits

1. **Isolation**: Tests don't affect real user configurations
2. **Reliability**: No interference between test runs
3. **Cleanup**: Automatic removal of test artifacts
4. **Safety**: Real provider configurations are preserved
5. **Debugging**: Easy to identify test vs. real providers

## Migration Guide

To update existing tests:

1. Replace direct provider name usage with prefixed versions using `get_test_provider_name()`
2. Add test utilities import: `use lc::test_with_cleanup;`
3. Wrap test body with `test_with_cleanup!({ ... })`
4. Update any assertions that check for specific provider names

### Example Migration

```rust
// Before
#[test]
fn test_provider_sync() {
    let config = create_config_with_providers();
    assert!(config.providers.contains_key("openai"));
}

// After
#[test]
fn test_provider_sync() {
    test_with_cleanup!({
        let config = create_config_with_providers();
        let openai_name = get_test_provider_name("openai");
        assert!(config.providers.contains_key(&openai_name));
    });
}
```

## Testing the System

Run the tests to verify the system works:

```bash
# Run specific test provider tests
cargo test test_provider_prefix

# Run all tests with cleanup
cargo test

# Check that cleanup works
cargo test test_setup_and_teardown_functions
```

## Troubleshooting

### Manual Cleanup

If test providers are left behind, you can manually clean them up:

```bash
# Navigate to the providers directory
cd ~/Library/Application\ Support/lc/providers

# List test providers
ls test-*

# Remove test providers
rm test-*
```

### Debugging

To see cleanup activity, the system prints messages when cleaning up files:

```
Cleaned up 3 test provider files
```

This helps verify that the cleanup system is working correctly.

## Notes

- The sync_commands.rs file contains cloud provider tests that are unrelated to the provider configuration system, so it doesn't need the test utilities
- Only tests that create or modify provider configurations in `~/Library/Application Support/lc/providers` need to use the test utilities
- The system is designed to be backward compatible - existing tests will continue to work without modification
- The common module has been updated to use prefixed provider names by default
- Integration tests in provider_commands.rs have been updated to use the new system