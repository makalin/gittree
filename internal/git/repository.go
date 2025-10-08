package git

import (
	"bufio"
	"context"
	"fmt"
	"os/exec"
	"strconv"
	"strings"
	"time"

	"github.com/go-git/go-git/v5"
	"github.com/go-git/go-git/v5/plumbing"
	"github.com/go-git/go-git/v5/plumbing/object"
)

// Repository represents a git repository
type Repository struct {
	repo *git.Repository
	path string
}

// Commit represents a git commit with graph information
type Commit struct {
	Hash        string            `json:"hash"`
	ShortHash   string            `json:"short_hash"`
	Message     string            `json:"message"`
	Author      string            `json:"author"`
	Email       string            `json:"email"`
	Date        time.Time         `json:"date"`
	Parents     []string          `json:"parents"`
	Refs        []string          `json:"refs"`
	Lane        int               `json:"lane"`
	Graph       []GraphLine       `json:"graph"`
	Files       []string          `json:"files"`
	Stats       map[string]int    `json:"stats"`
}

// GraphLine represents a line in the commit graph
type GraphLine struct {
	Type  GraphLineType `json:"type"`
	Lane  int           `json:"lane"`
	Merge bool          `json:"merge"`
}

// GraphLineType represents the type of graph line
type GraphLineType int

const (
	GraphLineNone GraphLineType = iota
	GraphLineVertical
	GraphLineHorizontal
	GraphLineCorner
	GraphLineMerge
)

// FilterOptions represents filtering options for commits
type FilterOptions struct {
	Author     string     `json:"author"`
	Path       string     `json:"path"`
	Since      *time.Time `json:"since"`
	Until      *time.Time `json:"until"`
	Range      string     `json:"range"`
	MaxCommits int        `json:"max_commits"`
}

// NewRepository creates a new repository instance
func NewRepository(path string) (*Repository, error) {
	repo, err := git.PlainOpen(path)
	if err != nil {
		return nil, err
	}

	return &Repository{
		repo: repo,
		path: path,
	}, nil
}

// GetCommits returns commits with graph information
func (r *Repository) GetCommits(ctx context.Context, filter FilterOptions) ([]*Commit, error) {
	// Build git log command
	args := []string{
		"log",
		"--graph",
		"--decorate=full",
		"--date-order",
		"--pretty=format:%H|%h|%an|%ae|%ad|%s|%P",
		"--date=iso",
	}

	// Add filters
	if filter.Author != "" {
		args = append(args, "--author", filter.Author)
	}
	if filter.Path != "" {
		args = append(args, "--", filter.Path)
	}
	if filter.Since != nil {
		args = append(args, "--since", filter.Since.Format("2006-01-02"))
	}
	if filter.Until != nil {
		args = append(args, "--until", filter.Until.Format("2006-01-02"))
	}
	if filter.Range != "" {
		args = append(args, filter.Range)
	}
	if filter.MaxCommits > 0 {
		args = append(args, "-n", strconv.Itoa(filter.MaxCommits))
	}

	// Execute git log
	cmd := exec.CommandContext(ctx, "git", args...)
	cmd.Dir = r.path
	output, err := cmd.Output()
	if err != nil {
		return nil, fmt.Errorf("failed to execute git log: %w", err)
	}

	// Parse output
	commits, err := r.parseGitLog(string(output))
	if err != nil {
		return nil, fmt.Errorf("failed to parse git log: %w", err)
	}

	// Generate graph
	if err := r.generateGraph(commits); err != nil {
		return nil, fmt.Errorf("failed to generate graph: %w", err)
	}

	// Get refs for each commit
	if err := r.addRefs(commits); err != nil {
		return nil, fmt.Errorf("failed to add refs: %w", err)
	}

	return commits, nil
}

// parseGitLog parses the output of git log --graph
func (r *Repository) parseGitLog(output string) ([]*Commit, error) {
	var commits []*Commit
	scanner := bufio.NewScanner(strings.NewReader(output))

	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())
		if line == "" {
			continue
		}

		// Parse graph characters and commit info
		parts := strings.Split(line, "|")
		if len(parts) < 7 {
			continue
		}

		graphStr := parts[0]
		hash := parts[1]
		shortHash := parts[2]
		author := parts[3]
		email := parts[4]
		dateStr := parts[5]
		message := parts[6]
		parentsStr := ""
		if len(parts) > 7 {
			parentsStr = parts[7]
		}

		// Parse date
		date, err := time.Parse("2006-01-02 15:04:05 -0700", dateStr)
		if err != nil {
			// Try alternative format
			date, err = time.Parse("2006-01-02 15:04:05 -07:00", dateStr)
			if err != nil {
				continue
			}
		}

		// Parse parents
		var parents []string
		if parentsStr != "" {
			parents = strings.Fields(parentsStr)
		}

		// Parse graph characters
		graph := r.parseGraphLine(graphStr)

		commit := &Commit{
			Hash:      hash,
			ShortHash: shortHash,
			Message:   message,
			Author:    author,
			Email:     email,
			Date:      date,
			Parents:   parents,
			Graph:     graph,
			Files:     []string{},
			Stats:     make(map[string]int),
		}

		commits = append(commits, commit)
	}

	return commits, nil
}

// parseGraphLine parses the graph characters from git log --graph
func (r *Repository) parseGraphLine(graphStr string) []GraphLine {
	var lines []GraphLine
	
	for i, char := range graphStr {
		line := GraphLine{Lane: i}
		
		switch char {
		case ' ':
			line.Type = GraphLineNone
		case '|', '*':
			line.Type = GraphLineVertical
		case '-', '_':
			line.Type = GraphLineHorizontal
		case '/', '\\':
			line.Type = GraphLineCorner
		case '+':
			line.Type = GraphLineMerge
			line.Merge = true
		default:
			line.Type = GraphLineNone
		}
		
		lines = append(lines, line)
	}
	
	return lines
}

// generateGraph generates the commit graph layout
func (r *Repository) generateGraph(commits []*Commit) error {
	if len(commits) == 0 {
		return nil
	}

	// Simple lane assignment based on graph characters
	for _, commit := range commits {
		if len(commit.Graph) > 0 {
			// Find the lane with a vertical line or merge
			for i, line := range commit.Graph {
				if line.Type == GraphLineVertical || line.Type == GraphLineMerge {
					commit.Lane = i
					break
				}
			}
		}
	}

	return nil
}

// addRefs adds ref information to commits
func (r *Repository) addRefs(commits []*Commit) error {
	// Get all refs
	refs, err := r.repo.References()
	if err != nil {
		return err
	}

	refMap := make(map[string][]string)
	err = refs.ForEach(func(ref *plumbing.Reference) error {
		if ref.Type() == plumbing.HashReference {
			hash := ref.Hash().String()
			name := ref.Name().String()
			refMap[hash] = append(refMap[hash], name)
		}
		return nil
	})
	if err != nil {
		return err
	}

	// Add refs to commits
	for _, commit := range commits {
		if refs, exists := refMap[commit.Hash]; exists {
			commit.Refs = refs
		}
	}

	return nil
}

// GetCommitDetails returns detailed information about a commit
func (r *Repository) GetCommitDetails(hash string) (*Commit, error) {
	commitHash := plumbing.NewHash(hash)
	commit, err := r.repo.CommitObject(commitHash)
	if err != nil {
		return nil, err
	}

	// Get file changes
	var files []string
	var stats = make(map[string]int)

	tree, err := commit.Tree()
	if err == nil {
		// Get parent tree for comparison
		var parentTree *object.Tree
		if len(commit.ParentHashes) > 0 {
			parentCommit, err := r.repo.CommitObject(commit.ParentHashes[0])
			if err == nil {
				parentTree, _ = parentCommit.Tree()
			}
		}

		// Compare trees
		changes, err := object.DiffTree(parentTree, tree)
		if err == nil {
			for _, change := range changes {
				if change.From.Name != "" {
					files = append(files, change.From.Name)
				}
				if change.To.Name != "" {
					files = append(files, change.To.Name)
				}
			}
		}
	}

	// Get parent hashes
	var parents []string
	for _, parentHash := range commit.ParentHashes {
		parents = append(parents, parentHash.String())
	}

	return &Commit{
		Hash:      commit.Hash.String(),
		ShortHash: commit.Hash.String()[:8],
		Message:   commit.Message,
		Author:    commit.Author.Name,
		Email:     commit.Author.Email,
		Date:      commit.Author.When,
		Parents:   parents,
		Files:     files,
		Stats:     stats,
	}, nil
}

// Checkout checks out a specific commit
func (r *Repository) Checkout(hash string) error {
	cmd := exec.Command("git", "checkout", hash)
	cmd.Dir = r.path
	return cmd.Run()
}

// ResetHard resets to a specific commit
func (r *Repository) ResetHard(hash string) error {
	cmd := exec.Command("git", "reset", "--hard", hash)
	cmd.Dir = r.path
	return cmd.Run()
}

// CherryPick cherry-picks a commit
func (r *Repository) CherryPick(hash string) error {
	cmd := exec.Command("git", "cherry-pick", hash)
	cmd.Dir = r.path
	return cmd.Run()
}

// Revert reverts a commit
func (r *Repository) Revert(hash string) error {
	cmd := exec.Command("git", "revert", hash)
	cmd.Dir = r.path
	return cmd.Run()
}

// CreateBranch creates a new branch at a commit
func (r *Repository) CreateBranch(name, hash string) error {
	cmd := exec.Command("git", "branch", name, hash)
	cmd.Dir = r.path
	return cmd.Run()
}

// CreateTag creates a new tag at a commit
func (r *Repository) CreateTag(name, hash string) error {
	cmd := exec.Command("git", "tag", name, hash)
	cmd.Dir = r.path
	return cmd.Run()
}

// GetCurrentBranch returns the current branch name
func (r *Repository) GetCurrentBranch() (string, error) {
	head, err := r.repo.Head()
	if err != nil {
		return "", err
	}
	return head.Name().Short(), nil
}

// IsDirty returns true if the working directory has uncommitted changes
func (r *Repository) IsDirty() (bool, error) {
	workTree, err := r.repo.Worktree()
	if err != nil {
		return false, err
	}
	
	status, err := workTree.Status()
	if err != nil {
		return false, err
	}
	
	return !status.IsClean(), nil
}