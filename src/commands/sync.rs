use crate::utils::git;

pub fn execute(args: &[String]) {
    if !args.is_empty() && (args[0] == "--help" || args[0] == "-h") {
        print_usage();
        return;
    }

    git::require_git_repo();

    let mut rebase = false;

    for arg in args {
        match arg.as_str() {
            "--rebase" => rebase = true,
            "--help" | "-h" => {
                print_usage();
                return;
            }
            flag if flag.starts_with('-') => {
                eprintln!("warning: unknown flag: {}", flag);
            }
            _ => {
                eprintln!("warning: unexpected argument '{}'", arg);
            }
        }
    }

    let pull_mode = if rebase { "--rebase" } else { "--ff-only" };

    if !git::has_upstream() {
        let branch = git::get_current_branch();
        eprintln!("No upstream branch configured for '{}'.", branch);
        eprintln!("Set upstream with: git push -u origin {}", branch);
        std::process::exit(1);
    }

    if git::has_uncommitted_changes() {
        println!("warning: you have uncommitted changes. Stash them first with 'git-helper stash'.");
    }

    git::step("Pulling latest changes...");
    match git::run(&["pull", pull_mode]) {
        git::GitResult::Failed(e) => {
            if rebase {
                eprintln!("error: pull (rebase) failed: {}", e);
                eprintln!("Resolve conflicts, then run 'git-helper sync' again.");
            } else {
                eprintln!("error: pull (ff-only) failed: {}", e);
                eprintln!("Try 'git-helper sync --rebase' or resolve manually.");
            }
            std::process::exit(1);
        }
        _ => {}
    }

    git::step("Pushing changes...");
    match git::run(&["push"]) {
        git::GitResult::Failed(e) => {
            eprintln!("error: push failed: {}", e);
            std::process::exit(1);
        }
        _ => {}
    }

    git::success("Sync complete. Local and remote are in sync.");
    if let Some((ahead, behind)) = git::count_ahead_behind() {
        println!("  ahead: {}, behind: {}", ahead, behind);
    }
}

fn print_usage() {
    println!("Usage: git-helper sync [options]");
    println!();
    println!("Pull latest changes and push in one command.");
    println!();
    println!("Options:");
    println!("  --rebase       Use rebase instead of merge when pulling");
    println!("  --help, -h     Show this help message");
    println!();
    println!("Examples:");
    println!("  git-helper sync");
    println!("  git-helper sync --rebase");
}
