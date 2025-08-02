# Cross-Platform PDF Text Extraction Backend Research

## Overview
This document provides research on Rust-based PDF libraries suitable for cross-platform PDF text extraction, comparing them against Python alternatives and evaluating licensing, performance, and compatibility considerations.

## Evaluated Libraries

### 1. pdf-extract (v0.9.0)
**Repository:** https://github.com/jrmuizel/pdf-extract  
**License:** MIT  
**Last Updated:** 4 months ago  
**Downloads:** 348,425 all-time, 58,001 recent  

#### Pros:
- ✅ **Simple API:** Straightforward text extraction with `extract_text_from_mem()` function
- ✅ **Pure Rust:** No external dependencies on system libraries
- ✅ **MIT License:** Very permissive, commercial-friendly
- ✅ **Well-adopted:** High download count indicates stability and trust
- ✅ **Active maintenance:** Recent updates (March 2024)
- ✅ **Cross-platform:** Works on all major platforms
- ✅ **Lightweight:** 94.7 KiB package size

#### Cons:
- ❌ **Basic functionality:** Limited to text extraction only
- ❌ **No advanced features:** No layout analysis, image extraction, or metadata
- ❌ **Dependency on lopdf:** Uses lopdf 0.36 internally, inheriting its limitations
- ❌ **Limited error handling:** Basic error reporting
- ❌ **No OCR support:** Cannot handle scanned PDFs

#### Dependencies:
- lopdf (0.36.0)
- log, simple_logger

---

### 2. lopdf (v0.36.0)
**Repository:** https://github.com/J-F-Liu/lopdf  
**License:** MIT  
**Last Updated:** 5 months ago  
**Downloads:** 2,148,157 all-time  

#### Pros:
- ✅ **Comprehensive PDF manipulation:** Full PDF creation, editing, and reading
- ✅ **MIT License:** Very permissive
- ✅ **Mature and stable:** Extremely high download count (2M+)
- ✅ **Pure Rust:** No system dependencies
- ✅ **Well-documented:** Extensive examples and documentation
- ✅ **Cross-platform:** Works everywhere
- ✅ **Low-level control:** Direct access to PDF objects and structure

#### Cons:
- ❌ **Complex API:** Requires understanding of PDF internals for advanced use
- ❌ **Text extraction is manual:** No high-level text extraction API
- ❌ **Learning curve:** Requires PDF specification knowledge
- ❌ **Large package:** 6.73 MiB (much larger than alternatives)
- ❌ **No OCR support:** Cannot handle scanned documents

#### Use Case:
- Better suited for PDF manipulation/creation rather than simple text extraction
- Would require building text extraction logic on top of the low-level API

---

### 3. pdfium-render (v0.8.34)
**Repository:** https://github.com/ajrcarey/pdfium-render  
**License:** Apache-2.0 or MIT  
**Last Updated:** Very recent (actively maintained)  
**Downloads:** Substantial user base  

#### Pros:
- ✅ **Google Pdfium backend:** Uses the same engine as Chrome
- ✅ **Comprehensive features:** Text extraction, rendering, form handling, annotations
- ✅ **High performance:** C++ backend with Rust safety
- ✅ **Dual license:** Apache-2.0 OR MIT (flexible)
- ✅ **Cross-platform:** Supports all major platforms including WASM
- ✅ **Rich API:** High-level idiomatic Rust interface
- ✅ **Active development:** Frequent updates and improvements
- ✅ **Thread safety:** Built-in thread safety mechanisms
- ✅ **Professional grade:** Used in production applications

#### Cons:
- ❌ **External dependency:** Requires Pdfium library (dynamic/static linking)
- ❌ **Complex setup:** Need to obtain Pdfium binaries separately
- ❌ **Large runtime:** Pdfium is a substantial library
- ❌ **Build complexity:** More complex build process
- ❌ **Licensing complexity:** Pdfium itself has different licensing

#### Dependencies:
- Requires Pdfium library (not included)
- Optional dependencies: image crate, libstdc++/libc++

---

### 4. extractous (v0.3.0)
**Repository:** https://github.com/yobix-ai/extractous  
**License:** Apache-2.0  
**Last Updated:** 7 months ago  
**Downloads:** 24,633 all-time, 18,149 recent  

#### Pros:
- ✅ **Multi-format support:** PDF, Word, Excel, HTML, and many others
- ✅ **Apache Tika backend:** Proven, enterprise-grade extraction
- ✅ **OCR support:** Built-in Tesseract integration
- ✅ **High-level API:** Simple, unified interface for all formats
- ✅ **Native performance:** Compiled Tika (no JVM runtime)
- ✅ **Comprehensive metadata:** Extracts both content and metadata
- ✅ **Streaming support:** Memory-efficient for large files

#### Cons:
- ❌ **Complex build requirements:** Requires GraalVM for native compilation
- ❌ **Heavy setup:** Tesseract required for OCR
- ❌ **Apache-2.0 only:** Less permissive than MIT
- ❌ **Large dependencies:** Apache Tika is substantial
- ❌ **Newer project:** Less mature than alternatives
- ❌ **Platform-specific builds:** GraalVM native image compilation challenges

#### Dependencies:
- GraalVM (build-time)
- Apache Tika (compiled to native)
- Tesseract (optional, for OCR)

---

## Comparison with Python Libraries

### Python Alternatives:
- **PyPDF2/PyPDF4:** Pure Python, limited features
- **pdfplumber:** Good for table extraction
- **pdfminer.six:** Comprehensive text extraction
- **pymupdf (fitz):** Fast, MuPDF backend
- **camelot:** Table-focused extraction

### Rust Advantages:
- ✅ **Performance:** Native compilation, no interpreter overhead
- ✅ **Memory safety:** Rust's ownership system prevents common bugs
- ✅ **Single binary:** No Python runtime dependency
- ✅ **Cross-compilation:** Easy to build for different targets
- ✅ **Smaller deployment:** No need to ship Python + dependencies

### Python Advantages:
- ✅ **Ecosystem maturity:** More specialized libraries available
- ✅ **Rapid prototyping:** Faster development cycle
- ✅ **Scientific libraries:** Better integration with ML/data science tools
- ✅ **Community:** Larger ecosystem for document processing

---

## Recommendations

### For Simple Text Extraction:
**Recommended: pdf-extract**
- Minimal setup, pure Rust
- Perfect for basic text extraction needs
- Reliable and well-maintained

### For Comprehensive PDF Processing:
**Recommended: pdfium-render**
- Professional-grade capabilities
- Excellent for complex PDFs with forms, annotations
- Worth the setup complexity for advanced use cases

### For Multi-Format Document Processing:
**Recommended: extractous**
- If you need to handle multiple document formats
- OCR support for scanned documents
- Higher setup complexity but more comprehensive

### For PDF Creation/Manipulation:
**Recommended: lopdf**
- Best choice if you need to create or modify PDFs
- Text extraction would require additional implementation

---

## Licensing Summary

| Library | License | Commercial Use | Copyleft |
|---------|---------|----------------|----------|
| pdf-extract | MIT | ✅ Yes | ❌ No |
| lopdf | MIT | ✅ Yes | ❌ No |
| pdfium-render | Apache-2.0 OR MIT | ✅ Yes | ❌ No |
| extractous | Apache-2.0 | ✅ Yes | ❌ No |

All evaluated libraries are commercial-friendly with permissive licenses.

---

## Final Recommendation

**For cross-platform PDF text extraction, I recommend starting with `pdf-extract`** for the following reasons:

1. **Simplicity:** Minimal setup, pure Rust implementation
2. **Reliability:** Well-tested, mature library with good adoption
3. **License:** MIT license is maximally permissive
4. **Maintenance:** Active development and recent updates
5. **Performance:** Native Rust performance without external dependencies

**Upgrade path:** If more advanced features are needed later, `pdfium-render` provides a comprehensive upgrade path while staying in the Rust ecosystem.

This approach follows the principle of starting simple and adding complexity only when needed.
