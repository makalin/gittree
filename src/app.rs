use crate::config::Config;
use crate::git::{FilterOptions, Repository};
use crate::simple_ui::SimpleApp;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;

pub struct App {
    repo: Repository,
    config: Config,
    filter: FilterOptions,
}

impl App {
    pub fn new(repo: Repository, config: Config, filter: FilterOptions) -> Self {
        Self {
            repo,
            config,
            filter,
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Load commits
        let commits = self.repo.get_commits(&self.filter)?;

        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Create and run the simple UI
        let mut ui_app = SimpleApp::new(self.repo.clone(), self.config.clone(), self.filter.clone(), commits);
        let result = ui_app.run();

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        result
    }
}