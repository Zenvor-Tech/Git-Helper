use crate::utils::git;
use colored::Colorize;
use std::io::{self, Write};
use std::path::Path;
use std::process::{self, Command};

pub fn execute(args: &[String]) {
    if args.is_empty() || args[0] == "--help" || args[0] == "-h" {
        print_usage();
        return;
    }

    if args[0] == "--ai" {
        cmd_ai_commit();
    } else {
        eprintln!(
            "{} unknown flag '{}' for commit.",
            "error:".red().bold(),
            args[0]
        );
        eprintln!("Usage: git-helper commit --ai");
        process::exit(1);
    }
}

fn cmd_ai_commit() {
    git::require_git_repo();

    let script_path = get_script_path();
    if !script_path.exists() {
        eprintln!(
            "{} AI commit script not found at {}",
            "error:".red().bold(),
            script_path.display()
        );
        eprintln!("Make sure scripts/ai_commit.py exists in the project.");
        process::exit(1);
    }

    let python = find_python();
    if python.is_none() {
        eprintln!("{} Python is required for AI commit. Install Python 3 and try again.", "error:".red().bold());
        process::exit(1);
    }
    let python = python.unwrap();

    git::step("Checking status...");
    Command::new("git")
        .args(&["add", "."])
        .stdout(process::Stdio::inherit())
        .stderr(process::Stdio::inherit())
        .output()
        .ok();

    if !git::has_uncommitted_changes() {
        eprintln!("No changes to commit. Make changes first.");
        process::exit(1);
    }

    git::step("Generating AI commit message...");
    let output = Command::new(&python)
        .arg(&script_path)
        .current_dir(&script_path.parent().unwrap())
        .output();

    let commit_msg = match output {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            eprintln!("{} AI commit generation failed:", "error:".red().bold());
            eprintln!("{}", stderr);
            process::exit(1);
        }
        Err(e) => {
            eprintln!("{} Failed to run AI script: {}", "error:".red().bold(), e);
            process::exit(1);
        }
    };

    if commit_msg.is_empty() {
        eprintln!("{} AI returned an empty commit message.", "error:".red().bold());
        process::exit(1);
    }

    println!();
    println!("{}", "Generated commit message:".bold().underline());
    println!("{}", "-".repeat(50));
    println!("{}", commit_msg);
    println!("{}", "-".repeat(50));
    println!();

    loop {
        print!("{} [Y/n/edit/e]: ", "Use this message?".cyan());
        io::stdout().flush().ok();
        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        let input = input.trim().to_lowercase();

        if input.is_empty() || input == "y" || input == "yes" {
            break;
        } else if input == "n" || input == "no" {
            println!("{}", "Commit cancelled.".yellow());
            process::exit(0);
        } else if input == "e" || input == "edit" {
            let edited = edit_message(&commit_msg);
            if let Some(msg) = edited {
                do_commit(&msg);
                return;
            }
            process::exit(1);
        } else {
            println!("Please answer Y, n, or e to edit.");
        }
    }

    do_commit(&commit_msg);
}

fn do_commit(message: &str) {
    git::step("Committing...");

    let mut cmd = Command::new("git");
    cmd.arg("commit").arg("-m").arg(message);

    let output = cmd
        .stdin(process::Stdio::inherit())
        .stdout(process::Stdio::inherit())
        .stderr(process::Stdio::inherit())
        .output();

    match output {
        Ok(out) if out.status.success() => {
            git::success("Commit created successfully.");
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            eprintln!("{} Commit failed: {}", "error:".red().bold(), stderr.trim());
            process::exit(1);
        }
        Err(e) => {
            eprintln!("{} {}", "error:".red().bold(), e);
            process::exit(1);
        }
    }
}

fn edit_message(original: &str) -> Option<String> {
    let tmp_path = std::env::temp_dir().join("git-helper-commit-msg.txt");
    if let Err(e) = std::fs::write(&tmp_path, original) {
        eprintln!("{} Could not create temp file: {}", "error:".red().bold(), e);
        return None;
    }

    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| {
            if cfg!(target_os = "windows") { "notepad".to_string() } else { "nano".to_string() }
        });

    println!("Opening editor...");
    let status = Command::new(&editor)
        .arg(&tmp_path)
        .stdin(process::Stdio::inherit())
        .stdout(process::Stdio::inherit())
        .stderr(process::Stdio::inherit())
        .status();

    match status {
        Ok(s) if s.success() => {
            let edited = std::fs::read_to_string(&tmp_path).ok()?;
            std::fs::remove_file(&tmp_path).ok();
            let edited = edited.trim().to_string();
            if edited.is_empty() || edited == original {
                eprintln!("Message unchanged.");
                return None;
            }
            Some(edited)
        }
        _ => {
            eprintln!("Editor closed without saving.");
            std::fs::remove_file(&tmp_path).ok();
            None
        }
    }
}

fn get_script_path() -> std::path::PathBuf {
    let repo_root = Command::new("git")
        .args(&["rev-parse", "--show-toplevel"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| {
            eprintln!("Not in a git repository.");
            process::exit(1);
        });

    Path::new(&repo_root).join("scripts").join("ai_commit.py")
}

fn find_python() -> Option<String> {
    for cmd in &["python3", "python"] {
        if let Ok(out) = Command::new(cmd).arg("--version").output() {
            if out.status.success() {
                return Some(cmd.to_string());
            }
        }
    }
    None
}

fn print_usage() {
    println!("Usage: git-helper commit --ai");
    println!();
    println!("Generate a commit message using AI.");
    println!();
    println!("Options:");
    println!("  --ai    Analyze staged changes and generate a commit message");
    println!("  --help, -h  Show this help message");
    println!();
    println!("Setup:");
    println!("  1. Copy .env.example to .env.local");
    println!("  2. Fill in your AI provider and API key");
    println!("  3. Run: git-helper commit --ai");
    println!();
    println!("Supported providers: openai, anthropic, gemini, groq, deepseek, openrouter, together, ollama");
    println!("Python 3 is required.");
}
