# Changelog

All notable changes to LLM Client (lc) will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Package distribution through major package managers (Homebrew, Scoop, APT, etc.)
- Comprehensive documentation website at https://lc.viwq.dev
- GitHub Actions workflows for automated releases
- Cross-platform testing matrix
- Logo and branding

### Changed
- Reorganized README to be more concise with links to full documentation
- Improved documentation structure with categorized sections

### Fixed
- Documentation site configuration for proper deployment

## [0.1.0] - 2025-01-XX

### Added
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

[Unreleased]: https://github.com/your-username/lc/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/your-username/lc/releases/tag/v0.1.0