mod commands;
mod config;
mod utils;

use colored::Colorize;
use std::process;
use utils::git;

const COMMANDS: &[&str] = &["save", "undo", "sync", "history", "log", "status", "st", "stash", "commit", "help", "version"];

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        commands::print_help();
        return;
    }

    let _cfg = config::Config::load();

    let result = match args[1].as_str() {
        "save" => commands::save::execute(&args[2..]),
        "undo" => commands::undo::execute(&args[2..]),
        "sync" => commands::sync::execute(&args[2..]),
        "history" | "log" => commands::history::execute(&args[2..]),
        "commit" => commands::commit::execute(&args[2..]),
        "status" | "st" => cmd_status(),
        "stash" => cmd_stash(&args[2..]),
        "help" | "--help" | "-h" => commands::print_help(),
        "version" | "--version" | "-v" => print_version(),
        cmd => {
            eprintln!(
                "{} unknown command '{}'.",
                "error:".red().bold(),
                cmd
            );
            if let Some(suggestion) = suggest_command(cmd) {
                eprintln!("{} Did you mean '{}'?", "hint:".cyan(), suggestion.cyan());
            }
            eprintln!("Use 'git-helper help' to see available commands.");
            process::exit(1);
        }
    };

    result
}

fn levenshtein(a: &str, b: &str) -> usize {
    let b_len = b.len();
    let mut prev: Vec<usize> = (0..=b_len).collect();
    let mut curr: Vec<usize> = vec![0; b_len + 1];

    for (i, ca) in a.chars().enumerate() {
        curr[0] = i + 1;
        for (j, cb) in b.chars().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            curr[j + 1] = (curr[j] + 1)
                .min(prev[j + 1] + 1)
                .min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b_len]
}

fn suggest_command(input: &str) -> Option<&'static str> {
    let threshold = if input.len() <= 3 { 1 } else { 2 };

    COMMANDS
        .iter()
        .filter(|&&c| c != input)
        .min_by_key(|&&c| levenshtein(input, c))
        .filter(|&&c| levenshtein(input, c) <= threshold)
        .copied()
}

fn cmd_status() {
    git::require_git_repo();

    let branch = git::get_current_branch();
    println!("{} {}", "On branch".bold(), branch.green().bold());

    if let Some((ahead, behind)) = git::count_ahead_behind() {
        if ahead > 0 {
            println!("  {} {} {}", "\u{2191}".cyan(), ahead, "commit(s) ahead".cyan());
        }
        if behind > 0 {
            println!("  {} {} {}", "\u{2193}".yellow(), behind, "commit(s) behind".yellow());
        }
        if ahead == 0 && behind == 0 {
            println!("  {} up to date with remote", "\u{2713}".green());
        }
    } else {
        println!("  {} no upstream branch", "\u{2716}".yellow());
    }

    println!("{}", "-".repeat(50));

    match git::run_capture(&["status", "--short"]) {
        Ok((out, _)) => {
            if out.is_empty() {
                println!("{} Working tree clean.", "\u{2713}".green());
            } else {
                for line in out.lines() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    let (status_chars, file) = if trimmed.len() > 2 {
                        (&trimmed[..2], trimmed[2..].trim())
                    } else {
                        (trimmed, "")
                    };
                    let staged = status_chars.chars().nth(0).unwrap_or(' ');
                    let working = status_chars.chars().nth(1).unwrap_or(' ');

                    let status_display = match (staged, working) {
                        ('?', '?') => format!("{}{}", staged, working).yellow(),
                        ('M', ' ') | (' ', 'M') | ('M', 'M') => format!("{}{}", staged, working).cyan(),
                        ('A', ' ') => format!("{}{}", staged, working).green(),
                        ('D', ' ') | (' ', 'D') => format!("{}{}", staged, working).red(),
                        ('R', ' ') => format!("{}{}", staged, working).magenta(),
                        _ => format!("{}{}", staged, working).normal(),
                    };
                    println!("{}  {}", status_display, file);
                }
            }
        }
        Err(e) => {
            eprintln!("error: {}", e);
        }
    }
}

fn cmd_stash(args: &[String]) {
    git::require_git_repo();

    if args.is_empty() {
        git::step("Stashing changes...");
        match git::run(&["stash", "push", "-m", "git-helper auto stash"]) {
            git::GitResult::Failed(e) => {
                eprintln!("error: stash failed: {}", e);
                process::exit(1);
            }
            _ => {
                git::success("Changes stashed.");
            }
        }
        return;
    }

    let subcommand = args[0].as_str();

    match subcommand {
        "pop" => {
            git::step("Restoring stashed changes...");
            match git::run(&["stash", "pop"]) {
                git::GitResult::Failed(e) => {
                    eprintln!("error: stash pop failed: {}", e);
                    process::exit(1);
                }
                _ => {
                    git::success("Stash popped.");
                }
            }
        }
        "list" | "ls" => {
            match git::run(&["stash", "list"]) {
                git::GitResult::Failed(e) => {
                    eprintln!("error: {}", e);
                    process::exit(1);
                }
                _ => {}
            }
        }
        "drop" => {
            git::step("Dropping latest stash...");
            match git::run(&["stash", "drop"]) {
                git::GitResult::Failed(e) => {
                    eprintln!("error: {}", e);
                    process::exit(1);
                }
                _ => {
                    git::success("Stash dropped.");
                }
            }
        }
        "apply" => {
            git::step("Applying stash...");
            match git::run(&["stash", "apply"]) {
                git::GitResult::Failed(e) => {
                    eprintln!("error: {}", e);
                    process::exit(1);
                }
                _ => {
                    git::success("Stash applied.");
                }
            }
        }
        "clear" => {
            eprintln!("warning: this will remove all stashes!");
            match git::run(&["stash", "clear"]) {
                git::GitResult::Failed(e) => {
                    eprintln!("error: {}", e);
                    process::exit(1);
                }
                _ => {
                    git::success("All stashes cleared.");
                }
            }
        }
        "--help" | "-h" => {
            print_stash_usage();
        }
        other => {
            eprintln!("unknown stash subcommand: '{}'", other);
            let stash_cmds = ["pop", "list", "ls", "drop", "apply", "clear"];
            let suggestion = stash_cmds
                .iter()
                .filter(|&&c| c != other)
                .min_by_key(|&&c| levenshtein(other, c))
                .filter(|&&c| levenshtein(other, c) <= 2)
                .copied();
            if let Some(s) = suggestion {
                eprintln!("{} Did you mean '{}'?", "hint:".cyan(), s.cyan());
            }
            print_stash_usage();
            process::exit(1);
        }
    }
}

fn print_stash_usage() {
    println!("Usage: git-helper stash [subcommand]");
    println!();
    println!("Manage stashed changes.");
    println!();
    println!("Subcommands:");
    println!("  (no subcommand)  Stash current changes");
    println!("  pop              Restore and remove latest stash");
    println!("  apply            Apply latest stash without removing it");
    println!("  list, ls         List all stashes");
    println!("  drop             Drop the latest stash");
    println!("  clear            Remove all stashes");
    println!("  --help, -h       Show this help message");
    println!();
    println!("Examples:");
    println!("  git-helper stash");
    println!("  git-helper stash pop");
    println!("  git-helper stash list");
}

fn print_version() {
    println!("git-helper v{}", env!("CARGO_PKG_VERSION"));
}
