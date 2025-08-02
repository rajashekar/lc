# LC File-Reading Architecture Audit

## Overview
This document audits the current file-reading architecture in the lc (LLM Client) Rust codebase to understand how attachments are handled and document the extension-based dispatch logic for adding PDF support.

## Current Architecture

### 1. Primary Attachment Handling (`src/cli.rs`)

The main attachment functionality is located in `src/cli.rs` with two key functions:

#### `read_and_format_attachments(attachments: &[String]) -> Result<String>`
- **Location**: `src/cli.rs:1729-1765`
- **Purpose**: Main function for reading and formatting file attachments
- **Current Logic**:
  1. Iterates through attachment file paths
  2. Uses `std::fs::read_to_string()` for text file reading
  3. Extracts file extension using `std::path::Path::extension()`
  4. Applies formatting based on file type
  5. Wraps content in `=== File: {path} ===` headers

#### `is_code_file(extension: &str) -> bool`
- **Location**: `src/cli.rs:1768-1777`
- **Purpose**: Extension-based dispatch logic for code file identification
- **Current Extensions Supported**:
  ```rust
  "rs" | "py" | "js" | "ts" | "java" | "cpp" | "c" | "h" | "hpp" |
  "go" | "rb" | "php" | "swift" | "kt" | "scala" | "sh" | "bash" |
  "zsh" | "fish" | "ps1" | "bat" | "cmd" | "html" | "css" | "scss" |
  "sass" | "less" | "xml" | "json" | "yaml" | "yml" | "toml" | "ini" |
  "cfg" | "conf" | "sql" | "r" | "m" | "mm" | "pl" | "pm" | "lua" |
  "vim" | "dockerfile" | "makefile" | "cmake" | "gradle" | "maven"
  ```

### 2. File Processing Pipeline

The attachment processing follows this flow:
1. **Command Line Parsing**: Attachments specified via `-a/--attach` flags (CLI definition in `src/cli.rs:60-61`)
2. **File Reading**: `read_and_format_attachments()` processes each file path
3. **Extension Detection**: Extracts file extension using `Path::extension()`
4. **Format Dispatch**: Uses `is_code_file()` to determine formatting:
   - **Code files**: Wrapped in markdown code blocks with language hint
   - **Other files**: Raw text content
5. **Content Assembly**: All files concatenated with headers and separators

### 3. Advanced File Processing (`src/vector_db.rs`)

The codebase also contains a more sophisticated file processing system in the vector database module:

#### `FileProcessor::is_text_file(path: &Path) -> bool`
- **Location**: `src/vector_db.rs:494-527`
- **Purpose**: More comprehensive file type detection
- **Logic**:
  - Extension-based categorization (text, code, binary)
  - Content analysis for files without extensions
  - Binary file exclusion list includes: `"pdf"` (line 512)

#### Extension Categories in `FileProcessor`:
- **Text files**: `"txt" | "md" | "markdown" | "rst" | "org" | "tex" | "rtf"`
- **Code files**: (Same as `is_code_file()` but more comprehensive)
- **Binary files to exclude**: 
  ```rust
  "exe" | "dll" | "so" | "dylib" | "bin" | "obj" | "o" | "a" | "lib" |
  "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" | "pdf" | "doc" |
  "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "jpg" | "jpeg" | "png" |
  "gif" | "bmp" | "tiff" | "svg" | "ico" | "mp3" | "mp4" | "avi" |
  "mov" | "wmv" | "flv" | "mkv" | "wav" | "flac" | "ogg"
  ```

### 4. Current Limitations for PDF Support

#### In Main Attachment System (`src/cli.rs`):
- Uses `std::fs::read_to_string()` which only works for UTF-8 text files
- PDF files would cause an error when passed to `read_to_string()`
- No binary file handling capability
- No content extraction for structured documents

#### In Vector Database System (`src/vector_db.rs`):
- PDF files are explicitly excluded from processing (line 512)
- System assumes all processable files are text-based

## Integration Points for PDF Support

### 1. Minimal Disruption Approach

To add PDF support with minimal changes to the existing architecture:

1. **Extend `is_code_file()` Logic**: 
   - Add a new companion function `is_binary_document(extension: &str) -> bool`
   - Handle PDF as a special case in `read_and_format_attachments()`

2. **Modify `read_and_format_attachments()`**:
   - Add PDF detection before the `std::fs::read_to_string()` call
   - Route PDF files to a new PDF processing function
   - Maintain the same output format (text with headers)

3. **New PDF Processing Function**:
   - Create `extract_pdf_text(file_path: &str) -> Result<String>`
   - Use a PDF parsing library (e.g., `pdf-extract`, `lopdf`, or `poppler`)
   - Return plain text content that fits into existing pipeline

### 2. Extension Points

#### File Type Detection Enhancement:
```rust
// Extend the existing pattern
pub fn is_pdf_file(extension: &str) -> bool {
    matches!(extension.to_lowercase().as_str(), "pdf")
}

pub fn requires_special_processing(extension: &str) -> bool {
    is_pdf_file(extension) // Extensible for other binary document types
}
```

#### Modified Processing Logic:
```rust
// In read_and_format_attachments()
let extension = std::path::Path::new(file_path)
    .extension()
    .and_then(|ext| ext.to_str())
    .unwrap_or("");

let content = if is_pdf_file(extension) {
    extract_pdf_text(file_path)?
} else {
    std::fs::read_to_string(file_path)
        .map_err(|e| anyhow!("Failed to read file '{}': {}", file_path, e))?
};
```

### 3. Vector Database Integration

For the embed command functionality, the PDF exclusion in `FileProcessor::is_text_file()` would need to be updated:

```rust
// In src/vector_db.rs, modify the binary files exclusion
// Remove "pdf" from the exclusion list or add special handling
match ext.as_str() {
    // ... existing cases ...
    "pdf" => true, // Now supported for text extraction
    // ... rest of binary exclusions ...
}
```

## Recommended Implementation Strategy

1. **Phase 1**: Add PDF text extraction to the main attachment system (`src/cli.rs`)
2. **Phase 2**: Integrate PDF support into the vector database file processor
3. **Phase 3**: Add support for other document formats using the same pattern

This approach maintains backward compatibility while adding PDF support through the existing extension-based dispatch mechanism.

## Dependencies Required

For PDF support, add to `Cargo.toml`:
```toml
[dependencies]
# Choose one:
pdf-extract = "0.7"  # Simple text extraction
# OR
lopdf = "0.32"       # More comprehensive PDF handling
# OR  
poppler = "0.2"      # System dependency on poppler
```

## Files to Modify

1. **`src/cli.rs`**: Main attachment processing logic
2. **`src/vector_db.rs`**: File type detection for embedding system
3. **`Cargo.toml`**: Add PDF processing dependency
4. **New module**: `src/pdf_processor.rs` (optional, for organization)
