# PDF Test Fixtures

This directory contains PDF test fixtures for automated testing of the PDF reader functionality.

## Test Files

### `simple_text.pdf`
- **Purpose**: Basic text extraction testing
- **Content**: Simple text document with plain ASCII text
- **Expected Result**: All text should be extracted correctly
- **Test Cases**: Basic functionality, text extraction accuracy

### `multi_page.pdf`
- **Purpose**: Multi-page document handling
- **Content**: Two-page PDF with different content on each page
- **Expected Result**: Text from both pages should be extracted, potentially with page separators
- **Test Cases**: Page handling, form feed character preservation

### `empty.pdf`
- **Purpose**: Edge case testing for empty content
- **Content**: Valid PDF structure but no text content
- **Expected Result**: Empty string or "[image page]" indicator
- **Test Cases**: Empty content handling, graceful degradation

### `corrupted.pdf`
- **Purpose**: Error handling testing
- **Content**: Invalid PDF file (plain text)
- **Expected Result**: Error with appropriate error message
- **Test Cases**: Error handling, graceful failure modes


## Test Strategy

### Unit Tests
- **File Reading**: Test reading from file paths, byte arrays, and readers
- **Error Handling**: Test various error conditions (file not found, corrupted files, etc.)
- **Feature Flags**: Test behavior with and without the `pdf` feature enabled
- **Thread Safety**: Verify thread-safe operations

### Integration Tests
- **Factory Pattern**: Test integration with the file reader factory
- **Extension Handling**: Test case-insensitive extension matching
- **End-to-End**: Test complete workflow from file to extracted text

### Performance Tests
- **Reader Creation**: Benchmark reader instantiation performance
- **Extension Checking**: Benchmark file extension validation
- **Memory Usage**: Monitor memory consumption during text extraction

## CI/CD Testing

The GitHub Actions workflow tests PDF functionality across multiple platforms:

### Test Matrix
- **Operating Systems**: Ubuntu (20.04, 22.04), macOS (12, 13), Windows (2019, 2022)
- **Rust Versions**: Minimum supported (1.70.0), stable
- **Architectures**: x86_64, aarch64 (macOS)

### Test Scenarios
1. **Feature Enabled**: Tests with `--features pdf`
2. **Feature Disabled**: Tests with `--no-default-features` to verify graceful degradation
3. **Cross-Platform**: Ensures consistent behavior across all supported platforms

## Expected Test Results

### Success Cases
- Simple text PDFs: Extract readable text content
- Multi-page PDFs: Extract text from all pages with proper separation
- Empty PDFs: Return empty string or appropriate indicator

### Error Cases
- Corrupted files: Return descriptive error messages
- Missing files: Return file not found errors
- Encrypted files: Return encryption/password errors
- Feature disabled: Return feature not enabled errors

### Performance Expectations
- Reader creation: < 100ms for 1000 instances
- Extension checking: < 100ms for 30000 checks
- Text extraction: Varies by PDF complexity and size

## Running the Tests

```bash
# Run all PDF tests with feature enabled
cargo test --features pdf pdf_reader_tests

# Run tests without PDF feature (error handling)
cargo test --no-default-features pdf_reader_tests::tests::test_pdf_feature_disabled

# Run specific test categories
cargo test --features pdf pdf_reader_tests::tests  # Unit tests
cargo test --features pdf pdf_reader_tests::integration_tests  # Integration tests
cargo test --features pdf pdf_reader_tests::performance_tests  # Performance tests

# Generate fixtures (if Python dependencies available)
python3 scripts/generate_pdf_fixtures.py
```

## Maintenance Notes

- Test fixtures should remain small and focused on specific test cases
- Manual PDF fixtures use minimal valid PDF structure for cross-platform compatibility
- Generated fixtures (via Python script) provide more realistic test scenarios
- Regular testing across all supported platforms ensures consistent behavior
- Performance benchmarks help detect regressions in PDF processing speed
