use crate::utils::git;

pub fn execute(args: &[String]) {
    if args.is_empty() || args[0] == "--help" || args[0] == "-h" {
        print_usage();
        return;
    }

    let mut skip_push = false;
    let mut amend = false;
    let message = parse_save_args(args, &mut skip_push, &mut amend);

    let message = match message {
        Some(m) => m,
        None => {
            eprintln!("error: no commit message provided");
            print_usage();
            std::process::exit(1);
        }
    };

    git::require_git_repo();

    git::step("Staging all changes...");
    if let Err(e) = match git::run(&["add", "."]) {
        git::GitResult::Failed(e) => Err(e),
        _ => Ok(()),
    } {
        eprintln!("error: failed to stage changes: {}", e);
        std::process::exit(1);
    }

    if !git::has_uncommitted_changes() {
        println!("No changes to commit.");
        return;
    }

    git::step("Committing...");
    let commit_args = if amend {
        vec!["commit", "--amend", "-m", &message]
    } else {
        vec!["commit", "-m", &message]
    };
    let commit_args_refs: Vec<&str> = commit_args.iter().map(|s| &**s).collect();
    if let git::GitResult::Failed(e) = git::run(&commit_args_refs) {
        eprintln!("error: commit failed: {}", e);
        std::process::exit(1);
    }

    if !skip_push {
        if git::has_upstream() {
            git::step("Pushing...");
            if let git::GitResult::Failed(e) = git::run(&["push"]) {
                eprintln!("warning: push failed: {}", e);
                println!("Commit succeeded but push failed. Use 'git-helper sync' to push later.");
                std::process::exit(1);
            }
        } else {
            println!("No upstream branch configured. Commit made locally.");
            println!("Set upstream with: git push -u origin {}", git::get_current_branch());
        }
    } else {
        println!("Skipping push (--skip-push).");
    }

    git::done();
}

fn parse_save_args(args: &[String], skip_push: &mut bool, amend: &mut bool) -> Option<String> {
    let mut msg_parts: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--skip-push" | "--no-push" => {
                *skip_push = true;
            }
            "--amend" => {
                *amend = true;
            }
            "--help" | "-h" => {
                return None;
            }
            flag if flag.starts_with('-') => {
                eprintln!("warning: unknown flag: {}", flag);
            }
            _ => {
                msg_parts.push(args[i].clone());
            }
        }
        i += 1;
    }

    if msg_parts.is_empty() {
        None
    } else {
        Some(msg_parts.join(" "))
    }
}

fn print_usage() {
    println!("Usage: git-helper save [options] <message>");
    println!();
    println!("Stage all changes, commit, and push to remote.");
    println!();
    println!("Options:");
    println!("  --skip-push    Commit locally without pushing");
    println!("  --amend        Amend the last commit instead of creating a new one");
    println!("  --help, -h     Show this help message");
    println!();
    println!("Examples:");
    println!("  git-helper save \"fix: resolve login bug\"");
    println!("  git-helper save --skip-push \"wip: refactoring module\"");
    println!("  git-helper save --amend \"fix: better commit message\"");
}
