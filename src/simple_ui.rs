use crate::config::Config;
use crate::git::{Commit, FilterOptions, Repository};
use std::io::{self, Write};

pub struct SimpleApp {
    repo: Repository,
    config: Config,
    filter: FilterOptions,
    commits: Vec<Commit>,
}

impl SimpleApp {
    pub fn new(repo: Repository, config: Config, filter: FilterOptions, commits: Vec<Commit>) -> Self {
        Self {
            repo,
            config,
            filter,
            commits,
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.commits.is_empty() {
            println!("No commits found");
            return Ok(());
        }

        println!("Git Graph - {} commits found", self.commits.len());
        println!("{}", "=".repeat(80));

        for (i, commit) in self.commits.iter().enumerate() {
            self.render_commit(commit, i);
        }

        println!("\nCommands:");
        println!("  q - Quit");
        println!("  h - Show help");
        println!("  c <hash> - Checkout commit");
        println!("  r <hash> - Reset to commit");
        println!("  p <hash> - Cherry-pick commit");
        println!("  b <name> - Create branch at current commit");
        println!("  t <name> - Create tag at current commit");
        println!("\nPress Enter to exit...");
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        let input = input.trim();
        if !input.is_empty() {
            self.handle_command(input)?;
        }

        Ok(())
    }

    fn render_commit(&self, commit: &Commit, index: usize) {
        // Render graph
        let graph = self.render_graph_line(commit);
        
        // Render commit info
        let info = format!(
            "{} {} {} {}",
            commit.short_hash,
            commit.author,
            commit.date.format(&self.config.date_format),
            commit.message
        );
        
        // Add refs
        let refs = if !commit.refs.is_empty() {
            format!(" ({})", commit.refs.join(", "))
        } else {
            String::new()
        };
        
        println!("{} {}{}", graph, info, refs);
    }

    fn render_graph_line(&self, commit: &Commit) -> String {
        if commit.graph.is_empty() {
            return "●".to_string();
        }
        
        let mut chars = Vec::new();
        for line in &commit.graph {
            let char = match line.line_type {
                crate::git::GraphLineType::Vertical => "│",
                crate::git::GraphLineType::Horizontal => "─",
                crate::git::GraphLineType::Corner => "└",
                crate::git::GraphLineType::Merge => "●",
                crate::git::GraphLineType::None => " ",
            };
            chars.push(char);
        }
        
        chars.join("")
    }

    fn handle_command(&self, input: &str) -> Result<(), Box<dyn std::error::Error>> {
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(());
        }

        match parts[0] {
            "q" => {
                println!("Goodbye!");
                std::process::exit(0);
            }
            "h" => {
                self.show_help();
            }
            "c" => {
                if parts.len() > 1 {
                    self.checkout_commit(parts[1])?;
                } else {
                    println!("Usage: c <hash>");
                }
            }
            "r" => {
                if parts.len() > 1 {
                    self.reset_commit(parts[1])?;
                } else {
                    println!("Usage: r <hash>");
                }
            }
            "p" => {
                if parts.len() > 1 {
                    self.cherry_pick_commit(parts[1])?;
                } else {
                    println!("Usage: p <hash>");
                }
            }
            "b" => {
                if parts.len() > 1 {
                    self.create_branch(parts[1])?;
                } else {
                    println!("Usage: b <name>");
                }
            }
            "t" => {
                if parts.len() > 1 {
                    self.create_tag(parts[1])?;
                } else {
                    println!("Usage: t <name>");
                }
            }
            _ => {
                println!("Unknown command: {}", parts[0]);
                println!("Type 'h' for help");
            }
        }

        Ok(())
    }

    fn show_help(&self) {
        println!("\nGit Tree Help:");
        println!("==============");
        println!("Commands:");
        println!("  q - Quit the application");
        println!("  h - Show this help");
        println!("  c <hash> - Checkout specific commit");
        println!("  r <hash> - Reset to specific commit (DANGEROUS)");
        println!("  p <hash> - Cherry-pick specific commit");
        println!("  b <name> - Create new branch at current commit");
        println!("  t <name> - Create new tag at current commit");
        println!("\nExamples:");
        println!("  c 99f7e7f  - Checkout commit 99f7e7f");
        println!("  b feature  - Create branch 'feature'");
        println!("  t v1.0.0  - Create tag 'v1.0.0'");
    }

    fn checkout_commit(&self, hash: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("Checking out commit {}...", hash);
        self.repo.checkout(hash)?;
        println!("Successfully checked out {}", hash);
        Ok(())
    }

    fn reset_commit(&self, hash: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("WARNING: This will reset your working directory to commit {}", hash);
        println!("Are you sure? (y/N)");
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        if input.trim().to_lowercase() == "y" {
            println!("Resetting to commit {}...", hash);
            self.repo.reset_hard(hash)?;
            println!("Successfully reset to {}", hash);
        } else {
            println!("Reset cancelled");
        }
        Ok(())
    }

    fn cherry_pick_commit(&self, hash: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("Cherry-picking commit {}...", hash);
        self.repo.cherry_pick(hash)?;
        println!("Successfully cherry-picked {}", hash);
        Ok(())
    }

    fn create_branch(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Get current commit (first in the list)
        if let Some(commit) = self.commits.first() {
            println!("Creating branch '{}' at commit {}...", name, commit.short_hash);
            self.repo.create_branch(name, &commit.hash)?;
            println!("Successfully created branch '{}'", name);
        } else {
            println!("No commits available");
        }
        Ok(())
    }

    fn create_tag(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Get current commit (first in the list)
        if let Some(commit) = self.commits.first() {
            println!("Creating tag '{}' at commit {}...", name, commit.short_hash);
            self.repo.create_tag(name, &commit.hash)?;
            println!("Successfully created tag '{}'", name);
        } else {
            println!("No commits available");
        }
        Ok(())
    }
}