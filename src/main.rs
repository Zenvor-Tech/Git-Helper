mod commands;
mod config;
mod utils;

use colored::Colorize;
use std::process;
use utils::git;

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
        "status" | "st" => cmd_status(),
        "stash" => cmd_stash(&args[2..]),
        "help" | "--help" | "-h" => commands::print_help(),
        "version" | "--version" | "-v" => print_version(),
        cmd => {
            eprintln!(
                "{} unknown command '{}'. Use 'git-helper help' to see available commands.",
                "error:".red().bold(),
                cmd
            );
            process::exit(1);
        }
    };

    result
}

fn cmd_status() {
    git::require_git_repo();

    let branch = git::get_current_branch();
    println!("{} {}", "On branch".bold(), branch.green().bold());

    if let Some((ahead, behind)) = git::count_ahead_behind() {
        if ahead > 0 || behind > 0 {
            println!(
                "  {} ahead {}, behind {}",
                "remote:".yellow(),
                ahead,
                behind
            );
        } else {
            println!("  {} up to date", "remote:".green());
        }
    } else {
        println!("  {} no upstream branch", "remote:".yellow());
    }

    println!("{}", "-".repeat(50));

    match git::run_capture(&["status", "--short"]) {
        Ok((out, _)) => {
            if out.is_empty() {
                println!("Working tree clean.");
            } else {
                for line in out.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with('?') {
                        println!("{}  {}", "?".yellow(), &line[1..].trim());
                    } else if trimmed.starts_with('M') {
                        println!("{}  {}", "M".cyan(), &line[1..].trim());
                    } else if trimmed.starts_with('A') {
                        println!("{}  {}", "A".green(), &line[1..].trim());
                    } else if trimmed.starts_with('D') {
                        println!("{}  {}", "D".red(), &line[1..].trim());
                    } else if trimmed.starts_with('R') {
                        println!("{}  {}", "R".magenta(), &line[1..].trim());
                    } else {
                        println!("{}", line);
                    }
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
