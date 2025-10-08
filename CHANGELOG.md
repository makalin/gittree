# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release
- Go backend using charm TUI library
- Rust backend using ratatui library
- GitHub-style commit graph visualization
- TUI interface with arrow key navigation
- Inline git operations (checkout, reset, cherry-pick, revert, branch, tag)
- Filtering by author, path, date, and range
- Configuration file support
- Unicode and ASCII graph modes
- LazyGit integration support
- Comprehensive documentation

### Features
- **GitHub-style graph**: Exact lane layout & merge bubbles (ASCII or Unicode)
- **TUI controls**: Arrow keys, vim keys, or mouse
- **Inline git ops**: `checkout`, `reset --hard`, `cherry-pick`, `revert`, `branch`, `tag`
- **Filters**: Author / path / date / head-only / PR-like ranges
- **Huge repo-ready**: Streaming log, virtualized viewport, caching
- **Drop-in tab for `lazygit`** *(bonus)*
- **Go (charm) or Rust (ratatui)** backends with identical UX