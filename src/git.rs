use chrono::{DateTime, Utc};
use git2::{Repository as Git2Repository, Oid};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub hash: String,
    pub short_hash: String,
    pub message: String,
    pub author: String,
    pub email: String,
    pub date: DateTime<Utc>,
    pub parents: Vec<String>,
    pub refs: Vec<String>,
    pub lane: usize,
    pub graph: Vec<GraphLine>,
    pub files: Vec<String>,
    pub stats: HashMap<String, i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphLine {
    pub line_type: GraphLineType,
    pub lane: usize,
    pub merge: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphLineType {
    None,
    Vertical,
    Horizontal,
    Corner,
    Merge,
}

#[derive(Debug, Clone)]
pub struct FilterOptions {
    pub author: Option<String>,
    pub path: Option<String>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub range: Option<String>,
    pub max_commits: Option<usize>,
}

#[derive(Clone)]
pub struct Repository {
    repo: Arc<Git2Repository>,
    path: String,
}

impl Repository {
    pub fn new(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let repo = Git2Repository::open(path)?;
        Ok(Self {
            repo: Arc::new(repo),
            path: path.to_string(),
        })
    }

    pub fn get_commits(&self, filter: &FilterOptions) -> Result<Vec<Commit>, Box<dyn std::error::Error>> {
        // Build git log command
        let mut args = vec![
            "log".to_string(),
            "--graph".to_string(),
            "--decorate=full".to_string(),
            "--date-order".to_string(),
            "--pretty=format:%H|%h|%an|%ae|%ad|%s|%P".to_string(),
            "--date=iso".to_string(),
        ];

        // Add filters
        if let Some(author) = &filter.author {
            args.extend(vec!["--author".to_string(), author.clone()]);
        }
        if let Some(path) = &filter.path {
            args.extend(vec!["--".to_string(), path.clone()]);
        }
        if let Some(since) = &filter.since {
            args.extend(vec!["--since".to_string(), since.format("%Y-%m-%d").to_string()]);
        }
        if let Some(until) = &filter.until {
            args.extend(vec!["--until".to_string(), until.format("%Y-%m-%d").to_string()]);
        }
        if let Some(range) = &filter.range {
            args.push(range.clone());
        }
        if let Some(max_commits) = &filter.max_commits {
            args.extend(vec!["-n".to_string(), max_commits.to_string()]);
        }

        // Execute git log
        let output = Command::new("git")
            .args(&args)
            .current_dir(&self.path)
            .output()?;

        if !output.status.success() {
            return Err(format!("git log failed: {}", String::from_utf8_lossy(&output.stderr)).into());
        }

        let output_str = String::from_utf8(output.stdout)?;
        let commits = self.parse_git_log(&output_str)?;

        // Generate graph
        self.generate_graph(&mut commits.clone())?;

        // Add refs
        self.add_refs(&mut commits.clone())?;

        Ok(commits)
    }

    fn parse_git_log(&self, output: &str) -> Result<Vec<Commit>, Box<dyn std::error::Error>> {
        let mut commits = Vec::new();

        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() < 7 {
                continue;
            }

            let graph_str = parts[0];
            let hash = parts[1].to_string();
            let short_hash = parts[2].to_string();
            let author = parts[3].to_string();
            let email = parts[4].to_string();
            let date_str = parts[5];
            let message = parts[6].to_string();
            let parents_str = if parts.len() > 7 { parts[7] } else { "" };

            // Parse date
            let date = chrono::DateTime::parse_from_rfc3339(date_str)
                .or_else(|_| chrono::DateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S %z"))
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            // Parse parents
            let parents: Vec<String> = if parents_str.is_empty() {
                Vec::new()
            } else {
                parents_str.split_whitespace().map(|s| s.to_string()).collect()
            };

            // Parse graph characters
            let graph = self.parse_graph_line(graph_str);

            let commit = Commit {
                hash,
                short_hash,
                message,
                author,
                email,
                date,
                parents,
                refs: Vec::new(),
                lane: 0,
                graph,
                files: Vec::new(),
                stats: HashMap::new(),
            };

            commits.push(commit);
        }

        Ok(commits)
    }

    fn parse_graph_line(&self, graph_str: &str) -> Vec<GraphLine> {
        let mut lines = Vec::new();

        for (i, ch) in graph_str.chars().enumerate() {
            let line_type = match ch {
                ' ' => GraphLineType::None,
                '|' | '*' => GraphLineType::Vertical,
                '-' | '_' => GraphLineType::Horizontal,
                '/' | '\\' => GraphLineType::Corner,
                '+' => GraphLineType::Merge,
                _ => GraphLineType::None,
            };

            let merge = ch == '+';

            lines.push(GraphLine {
                line_type,
                lane: i,
                merge,
            });
        }

        lines
    }

    fn generate_graph(&self, commits: &mut [Commit]) -> Result<(), Box<dyn std::error::Error>> {
        if commits.is_empty() {
            return Ok(());
        }

        // Simple lane assignment based on graph characters
        for commit in commits.iter_mut() {
            if !commit.graph.is_empty() {
                // Find the lane with a vertical line or merge
                for (i, line) in commit.graph.iter().enumerate() {
                    match line.line_type {
                        GraphLineType::Vertical | GraphLineType::Merge => {
                            commit.lane = i;
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }

    fn add_refs(&self, commits: &mut [Commit]) -> Result<(), Box<dyn std::error::Error>> {
        let mut ref_map: HashMap<String, Vec<String>> = HashMap::new();

        // Get all refs
        for reference in self.repo.references()? {
            let reference = reference?;
            if let Some(target) = reference.target() {
                let hash = target.to_string();
                let name = reference.name().unwrap_or("").to_string();
                ref_map.entry(hash).or_insert_with(Vec::new).push(name);
            }
        }

        // Add refs to commits
        for commit in commits.iter_mut() {
            if let Some(refs) = ref_map.get(&commit.hash) {
                commit.refs = refs.clone();
            }
        }

        Ok(())
    }

    pub fn get_commit_details(&self, hash: &str) -> Result<Commit, Box<dyn std::error::Error>> {
        let oid = Oid::from_str(hash)?;
        let commit = self.repo.find_commit(oid)?;

        // Get file changes
        let mut files = Vec::new();
        let stats = HashMap::new();

        if let Ok(tree) = commit.tree() {
            // Get parent tree for comparison
            let parent_tree = if let Ok(parent) = commit.parent(0) {
                parent.tree().ok()
            } else {
                None
            };

            // Compare trees
            if let Some(parent_tree) = parent_tree {
                if let Ok(diff) = self.repo.diff_tree_to_tree(Some(&parent_tree), Some(&tree), None) {
                    for delta in diff.deltas() {
                        if let Some(old_file) = delta.old_file().path() {
                            files.push(old_file.to_string_lossy().to_string());
                        }
                        if let Some(new_file) = delta.new_file().path() {
                            files.push(new_file.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }

        // Get parent hashes
        let parents: Vec<String> = (0..commit.parent_count())
            .filter_map(|i| commit.parent_id(i).ok())
            .map(|id| id.to_string())
            .collect();

        let author = commit.author();
        let author_name = author.name().unwrap_or("").to_string();
        let author_email = author.email().unwrap_or("").to_string();
        let author_when = author.when();
        
        Ok(Commit {
            hash: commit.id().to_string(),
            short_hash: commit.id().to_string()[..8].to_string(),
            message: commit.message().unwrap_or("").to_string(),
            author: author_name,
            email: author_email,
            date: DateTime::from_timestamp(author_when.seconds(), 0)
                .unwrap_or_else(|| Utc::now())
                .with_timezone(&Utc),
            parents,
            refs: Vec::new(),
            lane: 0,
            graph: Vec::new(),
            files,
            stats,
        })
    }

    pub fn checkout(&self, hash: &str) -> Result<(), Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .args(&["checkout", hash])
            .current_dir(&self.path)
            .output()?;

        if !output.status.success() {
            return Err(format!("git checkout failed: {}", String::from_utf8_lossy(&output.stderr)).into());
        }

        Ok(())
    }

    pub fn reset_hard(&self, hash: &str) -> Result<(), Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .args(&["reset", "--hard", hash])
            .current_dir(&self.path)
            .output()?;

        if !output.status.success() {
            return Err(format!("git reset --hard failed: {}", String::from_utf8_lossy(&output.stderr)).into());
        }

        Ok(())
    }

    pub fn cherry_pick(&self, hash: &str) -> Result<(), Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .args(&["cherry-pick", hash])
            .current_dir(&self.path)
            .output()?;

        if !output.status.success() {
            return Err(format!("git cherry-pick failed: {}", String::from_utf8_lossy(&output.stderr)).into());
        }

        Ok(())
    }

    pub fn revert(&self, hash: &str) -> Result<(), Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .args(&["revert", hash])
            .current_dir(&self.path)
            .output()?;

        if !output.status.success() {
            return Err(format!("git revert failed: {}", String::from_utf8_lossy(&output.stderr)).into());
        }

        Ok(())
    }

    pub fn create_branch(&self, name: &str, hash: &str) -> Result<(), Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .args(&["branch", name, hash])
            .current_dir(&self.path)
            .output()?;

        if !output.status.success() {
            return Err(format!("git branch failed: {}", String::from_utf8_lossy(&output.stderr)).into());
        }

        Ok(())
    }

    pub fn create_tag(&self, name: &str, hash: &str) -> Result<(), Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .args(&["tag", name, hash])
            .current_dir(&self.path)
            .output()?;

        if !output.status.success() {
            return Err(format!("git tag failed: {}", String::from_utf8_lossy(&output.stderr)).into());
        }

        Ok(())
    }

    pub fn get_current_branch(&self) -> Result<String, Box<dyn std::error::Error>> {
        let head = self.repo.head()?;
        let name = head.name().unwrap_or("HEAD");
        Ok(name.to_string())
    }

    pub fn is_dirty(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let mut status_options = git2::StatusOptions::new();
        status_options.include_ignored(false);
        status_options.include_untracked(true);
        
        let statuses = self.repo.statuses(Some(&mut status_options))?;
        Ok(!statuses.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_graph_line() {
        let repo = Repository {
            repo: unsafe { std::mem::zeroed() },
            path: "".to_string(),
        };

        let graph = repo.parse_graph_line("| | *");
        assert_eq!(graph.len(), 5);
        assert!(matches!(graph[0].line_type, GraphLineType::Vertical));
        assert!(matches!(graph[1].line_type, GraphLineType::None));
        assert!(matches!(graph[2].line_type, GraphLineType::Vertical));
        assert!(matches!(graph[3].line_type, GraphLineType::None));
        assert!(matches!(graph[4].line_type, GraphLineType::Vertical));
    }
}