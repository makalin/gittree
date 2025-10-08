package ui

import (
	"fmt"

	"github.com/makalin/gittree/internal/config"
	"github.com/makalin/gittree/internal/git"
)

// SimpleApp represents a simplified version of the app
type SimpleApp struct {
	repo   *git.Repository
	config *config.Config
	filter git.FilterOptions
}

// NewSimpleApp creates a new simple app instance
func NewSimpleApp(repo *git.Repository, cfg *config.Config, filter git.FilterOptions) *SimpleApp {
	return &SimpleApp{
		repo:   repo,
		config: cfg,
		filter: filter,
	}
}

// Run starts the simple application
func (a *SimpleApp) Run() error {
	// For now, just print a simple message
	fmt.Println("gittree - GitHub-like Git Graph")
	fmt.Println("This is a simplified version that compiles.")
	fmt.Println("Full TUI implementation coming soon...")
	
	// Load and display commits
	commits, err := a.repo.GetCommits(nil, a.filter)
	if err != nil {
		return err
	}
	
	fmt.Printf("Found %d commits\n", len(commits))
	for i, commit := range commits {
		if i >= 10 { // Limit to first 10 commits
			fmt.Println("...")
			break
		}
		fmt.Printf("%s %s %s\n", commit.ShortHash, commit.Author, commit.Message)
	}
	
	return nil
}