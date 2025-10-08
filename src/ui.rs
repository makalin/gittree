use crate::config::Config;
use crate::git::{Commit, FilterOptions, Repository};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::time::Duration;

pub struct App<'a> {
    repo: &'a Repository,
    config: Config,
    filter: FilterOptions,
    commits: Vec<Commit>,
    selected: usize,
    offset: usize,
    height: usize,
    width: usize,
    unicode: bool,
    show_help: bool,
    should_quit: bool,
}

impl<'a> App<'a> {
    pub fn new(repo: &'a Repository, config: Config, filter: FilterOptions, commits: Vec<Commit>) -> Self {
        let unicode = config.unicode;
        Self {
            repo,
            config,
            filter,
            commits,
            selected: 0,
            offset: 0,
            height: 0,
            width: 0,
            unicode,
            show_help: false,
            should_quit: false,
        }
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if crossterm::event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        self.handle_key_press(key.code)?;
                    }
                }
            }

            if self.should_quit {
                break;
            }
        }

        Ok(())
    }

    fn ui(&self, f: &mut Frame) {
        if self.show_help {
            self.render_help(f);
            return;
        }

        if self.commits.is_empty() {
            self.render_empty(f);
            return;
        }

        self.render_graph(f);
    }

    fn render_graph(&self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0)])
            .split(f.size());

        let items: Vec<ListItem> = self
            .commits
            .iter()
            .enumerate()
            .map(|(i, commit)| {
                let is_selected = i == self.selected;
                let style = if is_selected {
                    Style::default().add_modifier(Modifier::REVERSED)
                } else {
                    Style::default()
                };

                let graph = self.render_graph_line(commit);
                let info = format!(
                    "{} {} {} {}",
                    commit.short_hash,
                    commit.author,
                    commit.date.format(&self.config.date_format),
                    commit.message
                );

                let refs = if !commit.refs.is_empty() {
                    format!(" ({})", commit.refs.join(", "))
                } else {
                    String::new()
                };

                let line = format!("{} {}{}", graph, info, refs);
                ListItem::new(Line::from(Span::raw(line))).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Git Graph"));

        f.render_widget(list, chunks[0]);
    }

    fn render_graph_line(&self, commit: &Commit) -> String {
        if commit.graph.is_empty() {
            return "●".to_string();
        }

        let mut chars = Vec::new();
        for line in &commit.graph {
            let char = match line.line_type {
                crate::git::GraphLineType::Vertical => {
                    if self.unicode { "│" } else { "|" }
                }
                crate::git::GraphLineType::Horizontal => {
                    if self.unicode { "─" } else { "-" }
                }
                crate::git::GraphLineType::Corner => {
                    if self.unicode { "└" } else { "\\" }
                }
                crate::git::GraphLineType::Merge => "●",
                crate::git::GraphLineType::None => " ",
            };
            chars.push(char);
        }

        chars.join("")
    }

    fn render_help(&self, f: &mut Frame) {
        let help_text = r#"
gittree - GitHub-like Git Graph

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

Press ? to close this help.
"#;

        let paragraph = Paragraph::new(help_text)
            .block(Block::default().borders(Borders::ALL).title("Help"));

        f.render_widget(paragraph, f.size());
    }

    fn render_empty(&self, f: &mut Frame) {
        let paragraph = Paragraph::new("No commits found")
            .block(Block::default().borders(Borders::ALL).title("Git Graph"));

        f.render_widget(paragraph, f.size());
    }

    fn handle_key_press(&mut self, key: KeyCode) -> Result<(), Box<dyn std::error::Error>> {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
            }
            KeyCode::Char('?') => {
                self.show_help = !self.show_help;
            }
            KeyCode::Char('u') => {
                self.unicode = !self.unicode;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected < self.commits.len().saturating_sub(1) {
                    self.selected += 1;
                }
            }
            KeyCode::Left | KeyCode::Char('h') => {
                // Jump to parent
                if self.selected < self.commits.len() && !self.commits[self.selected].parents.is_empty() {
                    let parent_hash = &self.commits[self.selected].parents[0];
                    for (i, commit) in self.commits.iter().enumerate() {
                        if commit.hash == *parent_hash {
                            self.selected = i;
                            break;
                        }
                    }
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                // Jump to child
                if self.selected < self.commits.len() {
                    let current_hash = &self.commits[self.selected].hash;
                    for (i, commit) in self.commits.iter().enumerate() {
                        for parent in &commit.parents {
                            if parent == current_hash {
                                self.selected = i;
                                return Ok(());
                            }
                        }
                    }
                }
            }
            KeyCode::Char('g') => {
                self.selected = 0;
            }
            KeyCode::Char('G') => {
                self.selected = self.commits.len().saturating_sub(1);
            }
            KeyCode::PageUp => {
                if self.selected > 10 {
                    self.selected -= 10;
                } else {
                    self.selected = 0;
                }
            }
            KeyCode::PageDown => {
                if self.selected < self.commits.len().saturating_sub(10) {
                    self.selected += 10;
                } else {
                    self.selected = self.commits.len().saturating_sub(1);
                }
            }
            KeyCode::Enter => {
                if self.selected < self.commits.len() {
                    self.show_commit_details(&self.commits[self.selected])?;
                }
            }
            KeyCode::Char('c') => {
                if self.selected < self.commits.len() {
                    self.checkout_commit(&self.commits[self.selected])?;
                }
            }
            KeyCode::Char('x') => {
                if self.selected < self.commits.len() {
                    self.reset_to_commit(&self.commits[self.selected])?;
                }
            }
            KeyCode::Char('p') => {
                if self.selected < self.commits.len() {
                    self.cherry_pick_commit(&self.commits[self.selected])?;
                }
            }
            KeyCode::Char('r') => {
                if self.selected < self.commits.len() {
                    self.revert_commit(&self.commits[self.selected])?;
                }
            }
            KeyCode::Char('b') => {
                if self.selected < self.commits.len() {
                    self.create_branch(&self.commits[self.selected])?;
                }
            }
            KeyCode::Char('t') => {
                if self.selected < self.commits.len() {
                    self.create_tag(&self.commits[self.selected])?;
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn show_commit_details(&self, commit: &Commit) -> Result<(), Box<dyn std::error::Error>> {
        println!("Commit: {}", commit.hash);
        println!("Author: {} <{}>", commit.author, commit.email);
        println!("Date: {}", commit.date.format("%Y-%m-%d %H:%M:%S %Z"));
        println!("Message: {}", commit.message);
        println!("Parents: {}", commit.parents.join(", "));
        println!("Files: {}", commit.files.join(", "));
        Ok(())
    }

    fn checkout_commit(&self, commit: &Commit) -> Result<(), Box<dyn std::error::Error>> {
        if self.config.confirm_dangerous {
            println!("Checkout {}? (y/N): ", commit.short_hash);
        }
        self.repo.checkout(&commit.hash)
    }

    fn reset_to_commit(&self, commit: &Commit) -> Result<(), Box<dyn std::error::Error>> {
        if self.config.confirm_dangerous {
            println!("Reset to {}? (y/N): ", commit.short_hash);
        }
        self.repo.reset_hard(&commit.hash)
    }

    fn cherry_pick_commit(&self, commit: &Commit) -> Result<(), Box<dyn std::error::Error>> {
        self.repo.cherry_pick(&commit.hash)
    }

    fn revert_commit(&self, commit: &Commit) -> Result<(), Box<dyn std::error::Error>> {
        self.repo.revert(&commit.hash)
    }

    fn create_branch(&self, commit: &Commit) -> Result<(), Box<dyn std::error::Error>> {
        let branch_name = format!("branch-{}", commit.short_hash);
        self.repo.create_branch(&branch_name, &commit.hash)
    }

    fn create_tag(&self, commit: &Commit) -> Result<(), Box<dyn std::error::Error>> {
        let tag_name = format!("tag-{}", commit.short_hash);
        self.repo.create_tag(&tag_name, &commit.hash)
    }
}