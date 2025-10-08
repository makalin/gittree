# Installation Guide

## Prerequisites

- Go 1.22+ (for Go backend)
- Rust stable (for Rust backend)
- Git 2.0+

## Building from Source

### Go Backend

```bash
git clone https://github.com/makalin/gittree
cd gittree
make build-go
```

### Rust Backend

```bash
git clone https://github.com/makalin/gittree
cd gittree
make build-rs
```

### Both Backends

```bash
make build-all
```

## Installation

### Go Backend

```bash
make install-go
```

### Rust Backend

```bash
make install-rs
```

## Binary Releases

Prebuilt binaries are available in the [Releases](https://github.com/makalin/gittree/releases) section.

## Homebrew (macOS/Linux)

```bash
brew tap gittree/tap
brew install gittree
```

## Configuration

Create `~/.config/gittree/config.yml`:

```yaml
style: auto
unicode: true
dateFormat: "2006-01-02 15:04"
confirmDangerous: true
paging: "auto"
colors:
  graph1: "blue"
  graph2: "magenta"
  head: "cyan"
git:
  defaultRange: ""
  extraArgs: []
```

## LazyGit Integration

Add to `~/.config/lazygit/config.yml`:

```yaml
customCommands:
  - key: "T"
    context: "global"
    description: "Open GitHub-like graph (gittree)"
    command: "gittree --unicode"
    subprocess: true
```