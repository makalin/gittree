# Usage Guide

## Basic Usage

```bash
# In any git repository
gittree                 # full graph from HEAD
gittree --unicode       # fancy Unicode lanes
gittree -a "alice"      # filter by author
gittree --path src/     # path filter
gittree --since 2w      # time filter
gittree --range v1.2..  # rev range
```

## Keybindings

| Keys        | Action                                       |
| ----------- | -------------------------------------------- |
| ↑/k / ↓/j   | Move selection                               |
| ←/h / →/l   | Jump parents/children                        |
| PgUp / PgDn | Page                                         |
| g / G       | Top / Bottom                                 |
| Enter       | Open commit (details pane)                   |
| c           | Checkout selected (`git checkout <sha/ref>`) |
| x           | Reset to selected (`git reset --hard <sha>`) |
| p           | Cherry-pick selected                         |
| r           | Revert selected                              |
| b           | New branch at selected                       |
| t           | New tag at selected                          |
| /           | Filter (author/msg/path)                     |
| f           | Toggle follow file                           |
| u           | Toggle Unicode lanes                         |
| ?           | Help                                         |
| q           | Quit                                         |

## Command Line Options

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

## Examples

### View Recent Commits

```bash
gittree --since 1w
```

### Filter by Author

```bash
gittree --author "john"
```

### View Specific Path

```bash
gittree --path "src/"
```

### View Range

```bash
gittree --range "main..feature"
```

### Unicode Graph

```bash
gittree --unicode
```

### No Confirmation Prompts

```bash
gittree --yes
```

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

## Performance Tips

- Use `--path` to limit scope on large repositories
- Use `--range` to view specific branches
- Use `--max-commits` to limit the number of commits loaded
- Increase terminal width for better graph visualization
- Use a Unicode font for better graph characters

## Troubleshooting

### Graph looks different from GitHub
- Ensure `--date-order` is used (default)
- Try `--unicode` for better visualization

### Slow on monorepos
- Use `--path` to limit scope
- Use `--range` to view specific branches
- Increase terminal width

### Wide terminals
- Set `TERM` to a 256-color profile
- Use a Unicode font