package main

import (
	"flag"
	"fmt"
	"os"
	"time"

	"github.com/makalin/gittree/internal/config"
	"github.com/makalin/gittree/internal/git"
	"github.com/makalin/gittree/internal/ui"
	"github.com/charmbracelet/log"
)

var (
	version = "dev"
	commit  = "unknown"
	date    = "unknown"
)

func main() {
	var (
		unicode     = flag.Bool("unicode", false, "Use Unicode lane characters")
		noColor     = flag.Bool("no-color", false, "Disable colors")
		since       = flag.String("since", "", "Show commits more recent than a specific date")
		until       = flag.String("until", "", "Show commits older than a specific date")
		author      = flag.String("author", "", "Limit commits to author (regex)")
		path        = flag.String("path", "", "Limit commits to path (repeatable)")
		rangeFlag   = flag.String("range", "", "Rev range (e.g. main..feature)")
		maxCommits  = flag.Int("max-commits", 0, "Cap log read (0 = no limit)")
		yes         = flag.Bool("yes", false, "Skip confirmations")
		style       = flag.String("style", "auto", "Style (light, dark, auto)")
		showVersion = flag.Bool("version", false, "Show version information")
		help        = flag.Bool("help", false, "Show help")
	)

	flag.Parse()

	if *showVersion {
		fmt.Printf("gittree %s (commit: %s, built: %s)\n", version, commit, date)
		os.Exit(0)
	}

	if *help {
		showHelp()
		os.Exit(0)
	}

	// Load configuration
	cfg, err := config.Load()
	if err != nil {
		log.Warn("Failed to load config", "error", err)
		cfg = config.Default()
	}

	// Override config with command line flags
	if *unicode {
		cfg.Unicode = true
	}
	if *noColor {
		cfg.NoColor = true
	}
	if *style != "auto" {
		cfg.Style = *style
	}
	if *yes {
		cfg.ConfirmDangerous = false
	}

	// Parse date filters
	var sinceTime, untilTime *time.Time
	if *since != "" {
		if t, err := parseTime(*since); err == nil {
			sinceTime = &t
		} else {
			log.Fatal("Invalid since date", "error", err)
		}
	}
	if *until != "" {
		if t, err := parseTime(*until); err == nil {
			untilTime = &t
		} else {
			log.Fatal("Invalid until date", "error", err)
		}
	}

	// Create git repository instance
	repo, err := git.NewRepository(".")
	if err != nil {
		log.Fatal("Failed to open git repository", "error", err)
	}

	// Create filter options
	filter := git.FilterOptions{
		Author:     *author,
		Path:       *path,
		Since:      sinceTime,
		Until:      untilTime,
		Range:      *rangeFlag,
		MaxCommits: *maxCommits,
	}

	// Create and run the TUI
	app := ui.NewSimpleApp(repo, cfg, filter)
	if err := app.Run(); err != nil {
		log.Fatal("Application error", "error", err)
	}
}

func parseTime(s string) (time.Time, error) {
	// Try common formats
	formats := []string{
		time.RFC3339,
		"2006-01-02",
		"2006-01-02 15:04:05",
		"2w", "1w", "3d", "2d", "1d", "12h", "6h", "1h",
	}

	for _, format := range formats {
		if t, err := time.Parse(format, s); err == nil {
			return t, nil
		}
	}

	// Handle relative time formats
	now := time.Now()
	switch s {
	case "2w":
		return now.AddDate(0, 0, -14), nil
	case "1w":
		return now.AddDate(0, 0, -7), nil
	case "3d":
		return now.AddDate(0, 0, -3), nil
	case "2d":
		return now.AddDate(0, 0, -2), nil
	case "1d":
		return now.AddDate(0, 0, -1), nil
	case "12h":
		return now.Add(-12 * time.Hour), nil
	case "6h":
		return now.Add(-6 * time.Hour), nil
	case "1h":
		return now.Add(-1 * time.Hour), nil
	}

	return time.Time{}, fmt.Errorf("unable to parse time: %s", s)
}

func showHelp() {
	fmt.Println(`gittree - Local GitHub-Like Graph

USAGE:
    gittree [OPTIONS]

OPTIONS:
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
    --version           Show version information
    --help              Show this help message

KEYBINDINGS:
    ↑/k / ↓/j          Move selection
    ←/h / →/l          Jump parents/children
    PgUp / PgDn        Page
    g / G              Top / Bottom
    Enter              Open commit (details pane)
    c                  Checkout selected
    x                  Reset to selected
    p                  Cherry-pick selected
    r                  Revert selected
    b                  New branch at selected
    t                  New tag at selected
    /                  Filter (author/msg/path)
    f                  Toggle follow file
    u                  Toggle Unicode lanes
    ?                  Help
    q                  Quit

EXAMPLES:
    gittree                    # full graph from HEAD
    gittree --unicode          # fancy Unicode lanes
    gittree -a "alice"         # filter by author
    gittree --path src/        # path filter
    gittree --since 2w         # time filter
    gittree --range v1.2..     # rev range
`)
}