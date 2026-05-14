use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub struct Config {
    pub default_push: bool,
    pub default_rebase: bool,
    pub history_count: usize,
    pub auto_fetch: bool,
    settings: HashMap<String, String>,
}

impl Config {
    pub fn load() -> Self {
        let path = get_config_path();
        let mut config = Config::default();

        if let Some(path) = path {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    config.parse(&content);
                    println!("Loaded config from {}", path.display());
                }
            }
        }

        config
    }

    fn default() -> Self {
        let mut settings = HashMap::new();
        settings.insert("default_push".to_string(), "true".to_string());
        settings.insert("default_rebase".to_string(), "false".to_string());
        settings.insert("history_count".to_string(), "20".to_string());
        settings.insert("auto_fetch".to_string(), "false".to_string());

        Config {
            default_push: true,
            default_rebase: false,
            history_count: 20,
            auto_fetch: false,
            settings,
        }
    }

    fn parse(&mut self, content: &str) {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//") {
                continue;
            }

            if let Some(eq_pos) = trimmed.find('=') {
                let key = trimmed[..eq_pos].trim().to_lowercase();
                let value = trimmed[eq_pos + 1..].trim().to_string();
                let value_lower = value.to_lowercase();

                self.settings.insert(key.clone(), value.clone());

                match key.as_str() {
                    "default_push" => self.default_push = value_lower == "true",
                    "default_rebase" => self.default_rebase = value_lower == "true",
                    "history_count" => {
                        if let Ok(n) = value.parse() {
                            self.history_count = n;
                        }
                    }
                    "auto_fetch" => self.auto_fetch = value_lower == "true",
                    _ => {}
                }
            }
        }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.settings.get(key).map(|s| s.as_str())
    }
}

fn get_config_path() -> Option<PathBuf> {
    if let Ok(home) = std::env::var("HOME") {
        let path = PathBuf::from(home).join(".git-helper").join("config");
        return Some(path);
    }

    if let Ok(home) = std::env::var("USERPROFILE") {
        let path = PathBuf::from(home).join(".git-helper").join("config");
        return Some(path);
    }

    None
}

pub fn get_git_config(key: &str) -> Option<String> {
    let output = std::process::Command::new("git")
        .args(&["config", "--get", key])
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}
