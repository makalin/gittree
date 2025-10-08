use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub style: String,
    pub unicode: bool,
    pub no_color: bool,
    pub date_format: String,
    pub confirm_dangerous: bool,
    pub paging: String,
    pub colors: Colors,
    pub git: GitConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Colors {
    pub graph1: String,
    pub graph2: String,
    pub head: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    pub default_range: String,
    pub extra_args: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            style: "auto".to_string(),
            unicode: false,
            no_color: false,
            date_format: "%Y-%m-%d %H:%M".to_string(),
            confirm_dangerous: true,
            paging: "auto".to_string(),
            colors: Colors {
                graph1: "blue".to_string(),
                graph2: "magenta".to_string(),
                head: "cyan".to_string(),
            },
            git: GitConfig {
                default_range: String::new(),
                extra_args: Vec::new(),
            },
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = get_config_path()?;
        
        if !config_path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(config_path)?;
        let config: Config = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = get_config_path()?;
        
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_yaml::to_string(self)?;
        fs::write(config_path, content)?;
        Ok(())
    }
}

fn get_config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home = std::env::var("HOME")?;
    Ok(PathBuf::from(home).join(".config").join("gittree").join("config.yml"))
}