use colored::Colorize;
use std::process::{self, Command};

pub enum GitResult {
    Success,
    Failed(String),
}

pub fn run(args: &[&str]) -> GitResult {
    let output = Command::new("git")
        .args(args)
        .stdin(process::Stdio::inherit())
        .stdout(process::Stdio::inherit())
        .stderr(process::Stdio::inherit())
        .output();

    match output {
        Ok(out) if out.status.success() => GitResult::Success,
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            GitResult::Failed(stderr.trim().to_string())
        }
        Err(e) => GitResult::Failed(format!("Failed to run git: {}", e)),
    }
}

pub fn run_capture(args: &[&str]) -> Result<(String, String), String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run git: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if output.status.success() {
        Ok((stdout, stderr))
    } else {
        Err(stderr)
    }
}

pub fn require_git_repo() {
    match run_capture(&["rev-parse", "--git-dir"]) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{} {}", "error:".red().bold(), e);
            eprintln!("Not a git repository. Run this command inside a git repository.");
            process::exit(1);
        }
    }
}

pub fn get_current_branch() -> String {
    match run_capture(&["rev-parse", "--abbrev-ref", "HEAD"]) {
        Ok((branch, _)) => branch,
        Err(_) => "unknown".to_string(),
    }
}

pub fn has_upstream() -> bool {
    run_capture(&["rev-parse", "--abbrev-ref", "@{upstream}"]).is_ok()
}

pub fn has_uncommitted_changes() -> bool {
    match run_capture(&["status", "--porcelain"]) {
        Ok((out, _)) => !out.is_empty(),
        Err(_) => false,
    }
}

pub fn count_ahead_behind() -> Option<(usize, usize)> {
    let branch = get_current_branch();
    if !has_upstream() {
        return None;
    }
    let range = format!("{}...{}@{{u}}", branch, branch);
    match run_capture(&["rev-list", "--left-right", "--count", &range]) {
        Ok((out, _)) => {
            let parts: Vec<&str> = out.split_whitespace().collect();
            if parts.len() == 2 {
                let ahead = parts[0].parse().unwrap_or(0);
                let behind = parts[1].parse().unwrap_or(0);
                Some((ahead, behind))
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

pub fn step(message: &str) {
    println!("{} {}", "▶".cyan().bold(), message);
}

pub fn success(message: &str) {
    println!("{} {}", "✔".green().bold(), message);
}

pub fn done() {
    println!("{}", " Done!".green().bold());
}
