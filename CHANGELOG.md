# Changelog

All notable changes to LLM Client (lc) will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- System dependencies documentation and build requirements for pkg-config, OpenSSL
- Comprehensive troubleshooting guide for OpenSSL and pkg-config build errors
- Automated system dependency installation in GitHub Actions workflows for reliable CI/CD
- Package distribution through major package managers (Homebrew, Scoop, APT, etc.)
- Comprehensive documentation website at https://lc.viwq.dev
- GitHub Actions workflows for automated releases
- Cross-platform testing matrix
- Logo and branding
- Comprehensive tests to ensure capability flags match explicit provider data

### Changed
- **BREAKING**: Stricter capability display policy for model icons
  - Capability icons (🔧 tools, 👁 vision, 🧠 reasoning, 💻 code, etc.) are now only displayed when explicitly provided by the provider's API response
  - Removed capability inference based on model names or patterns
  - Users may notice some previously visible icons have disappeared - this doesn't mean models lost capabilities, just that providers don't explicitly advertise them
- Reorganized README to be more concise with links to full documentation
- Improved documentation structure with categorized sections
- Updated model metadata extraction to only use explicit capability data from JSON responses

### Fixed
- Documentation site configuration for proper deployment
- Eliminated assumptions in capability detection that could lead to misleading displays
- Streaming output now appears in real time (disabled gzip on streaming requests)

## [0.1.0] - 2025-01-XX

### Added
- System dependencies documentation and build requirements for pkg-config, OpenSSL
- Comprehensive troubleshooting guide for OpenSSL and pkg-config build errors
- Automated system dependency installation in GitHub Actions workflows for reliable CI/CD
- Initial release of LLM Client
- Core features:
  - Provider management for any OpenAI-compatible API
  - Direct prompts and interactive chat sessions
  - SQLite-based conversation logging
  - API key management
  - Configuration management
- Advanced features:
  - Built-in vector database with embeddings
  - Similarity search functionality
  - RAG (Retrieval-Augmented Generation) support
  - File embedding with intelligent chunking
  - Configuration sync with encryption
- Command-line interface with short aliases
- Support for multiple response formats (OpenAI, Anthropic, Cohere, etc.)
- Custom headers support for providers
- Models command with rich filtering options
- Cross-platform support (Linux, macOS, Windows)

### Performance
- ~3ms cold start time
- ~6MB memory usage
- ~8MB binary size

[Unreleased]: https://github.com/rajashekar/lc/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/rajashekar/lc/releases/tag/v0.1.0