# gittree — Local GitHub-Like Graph

[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](#license)
[![Go](https://img.shields.io/badge/Go-%3E=1.22-00ADD8)](#go-install)
[![Rust](https://img.shields.io/badge/Rust-stable-DEA584)](#rust-install)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](#contributing)

> **Problem:** `git log --graph` becomes unreadable on large repos.
> **Solution:** A fast TUI that renders an ASCII/Unicode commit tree like **GitHub’s network graph**, with arrow-key navigation and inline actions (checkout, reset, cherry-pick).

---

## Highlights

* **GitHub-style graph**: Exact lane layout & merge bubbles (ASCII or Unicode).
* **TUI controls**: Arrow keys, vim keys, or mouse.
* **Inline git ops**: `checkout`, `reset --hard`, `cherry-pick`, `revert`, `branch`, `tag`.
* **Filters**: Author / path / date / head-only / PR-like ranges.
* **Huge repo-ready**: Streaming log, virtualized viewport, caching.
* **Drop-in tab for `lazygit`** *(bonus)*.
* **Go (charm) or Rust (ratatui)** backends with identical UX.

---

## Demo

```
o──● main  Merge feature/auth                (You)  2h
│  │\
│  │ ● fix: race in session store                3h
│  ●─┘
│  ● feat(auth): add OIDC                         5h
│ /
●─┘ chore: bump deps                              1d
```

*(Screenshots: `docs/screenshot-tty.png`, `docs/screenshot-unicode.png`)*

---

## Install

### macOS/Linux (Homebrew)

```bash
brew tap gittree/tap
brew install gittree
```

*(If the tap isn’t available yet, use Go/Rust install below.)*

### <a id="go-install"></a>Go (charm)

```bash
go install github.com/makalin/gittree/cmd/gittree@latest
```

### <a id="rust-install"></a>Rust (ratatui)

```bash
cargo install gittree
```

### Binary releases

Prebuilt binaries for macOS, Linux, Windows: see **Releases**.

---

## Usage

```bash
# In any git repo
gittree                 # full graph from HEAD
gittree --unicode       # fancy Unicode lanes
gittree -a "alice"      # filter by author
gittree --path src/     # path filter
gittree --since 2w      # time filter
gittree --range v1.2..  # rev range
```

### Keybindings

| Keys        | Action                                       |        |
| ----------- | -------------------------------------------- | ------ |
| ↑/k / ↓/j   | Move selection                               |        |
| ←/h / →/l   | Jump parents/children                        |        |
| PgUp / PgDn | Page                                         |        |
| g / G       | Top / Bottom                                 |        |
| Enter       | Open commit (details pane)                   |        |
| c           | Checkout selected (`git checkout <sha        | ref>`) |
| x           | Reset to selected (`git reset --hard <sha>`) |        |
| p           | Cherry-pick selected                         |        |
| r           | Revert selected                              |        |
| b           | New branch at selected                       |        |
| t           | New tag at selected                          |        |
| /           | Filter (author/msg/path)                     |        |
| f           | Toggle follow file                           |        |
| u           | Toggle Unicode lanes                         |        |
| ?           | Help                                         |        |
| q           | Quit                                         |        |

> Destructive actions (reset) prompt for confirmation unless `--yes`.

---

## Commit Details Pane

* Full message, diffstat, parents, refs, files changed, and patch preview.
* Press `o` to open in `$PAGER`, `O` to open in your GUI diff tool.

---

## Flags

```
--unicode           Use Unicode lane characters
--no-color          Disable colors
--since, --until    Date filters (e.g. 2025-01-01, 2w, 48h)
--author            Author regex
--path              Limit to path (repeatable)
--range             Rev range (e.g. main..feature)
--max-commits N     Cap log read
--pager             Use $PAGER for details
--yes               Skip confirmations
--backend {go,rs}   Force backend
--style {light,dark,auto}
```

---

## LazyGit Integration (Bonus)

Add a custom command tab to `~/.config/lazygit/config.yml`:

```yaml
customCommands:
  - key: "T"
    context: "global"
    description: "Open GitHub-like graph (gittree)"
    command: "gittree --unicode"
    subprocess: true
```

Or embed as a panel by using `gittree --range {{.SelectedBranch}}.. --no-color`.

---

## Performance

* **Streaming parser** over `git log --graph --decorate=full --date-order`.
* **Lane engine** matches GitHub’s bundling of merges & branch tips.
* **Virtual list** keeps memory flat for 100k+ commits.
* Optional **graph cache** at `.git/.gittree-cache`.

---

## Configuration

Create `~/.config/gittree/config.yml`:

```yaml
style: auto           # light | dark | auto
unicode: true
dateFormat: "2006-01-02 15:04"
confirmDangerous: true
paging: "auto"        # auto | always | never
colors:
  graph1: "blue"
  graph2: "magenta"
  head:   "cyan"
git:
  defaultRange: ""
  extraArgs: []
```

---

## Building From Source

### Go backend

```bash
git clone https://github.com/makalin/gittree
cd gittree
make build-go   # or: go build ./cmd/gittree
```

### Rust backend

```bash
git clone https://github.com/makalin/gittree
cd gittree
make build-rs   # or: cargo build --release
```

---

## Roadmap

* [ ] Stashes & reflog lanes
* [ ] PR/issue linking via remote provider hints
* [ ] Partial clones & worktrees awareness
* [ ] Interactive rebase view
* [ ] Inline blame overlay for selected file

---

## Troubleshooting

* **Graph looks different from GitHub** → Ensure `--date-order` is used (default). Try `--unicode`.
* **Slow on monorepos** → Use `--path`, `--range`, or increase terminal width.
* **Wide terminals** → Set `TERM` to a 256-color profile; use a Unicode font.

---

## Alternatives

* `git log --graph` (built-in, minimal)
* `tig` (powerful, not GitHub-like lanes)
* `gitui`, `lazygit` (general GUIs; integrate `gittree` as a tab)

---

## Contributing

Issues and PRs welcome! Please run linters and add tests:

```bash
make lint test
```

---

## License

MIT © Mehmet T. AKALIN. See [LICENSE](LICENSE).
