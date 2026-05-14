use crate::utils::git;

pub fn execute(args: &[String]) {
    if !args.is_empty() && (args[0] == "--help" || args[0] == "-h") {
        print_usage();
        return;
    }

    git::require_git_repo();

    let mut hard = false;
    let mut count: usize = 1;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--hard" => {
                hard = true;
            }
            "--count" | "-n" => {
                i += 1;
                if i < args.len() {
                    count = args[i].parse().unwrap_or_else(|_| {
                        eprintln!("error: invalid count '{}'", args[i]);
                        std::process::exit(1);
                    });
                } else {
                    eprintln!("error: --count requires a number");
                    std::process::exit(1);
                }
            }
            flag if flag.starts_with('-') => {
                eprintln!("warning: unknown flag: {}", flag);
            }
            _ => {
                count = args[i].parse().unwrap_or_else(|_| {
                    eprintln!("error: invalid count '{}'", args[i]);
                    std::process::exit(1);
                });
            }
        }
        i += 1;
    }

    if count == 0 {
        eprintln!("error: count must be greater than 0");
        std::process::exit(1);
    }

    let mode = if hard { "--hard" } else { "--soft" };
    let target = format!("HEAD~{}", count);

    let action = if hard { "Discarding" } else { "Undoing" };
    let mode_desc = if hard {
        "changes will be DISCARDED"
    } else {
        "keeping changes staged"
    };

    println!("{} {} last commit(s) ({})", "▶", action, mode_desc);
    println!("  reset {} {}", mode, target);

    match git::run(&["reset", mode, &target]) {
        git::GitResult::Success => {
            if hard {
                git::success(&format!("Removed last {} commit(s).", count));
            } else {
                git::success(&format!(
                    "Last {} commit(s) undone. Changes are staged and ready to edit.",
                    count
                ));
            }
        }
        git::GitResult::Failed(e) => {
            eprintln!("error: undo failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn print_usage() {
    println!("Usage: git-helper undo [options] [count]");
    println!();
    println!("Undo recent commits while keeping changes staged.");
    println!();
    println!("Options:");
    println!("  --hard         Discard changes completely (irreversible!)");
    println!("  --count <N>    Undo N commits (default: 1)");
    println!("  -n <N>         Same as --count");
    println!("  --help, -h     Show this help message");
    println!();
    println!("Examples:");
    println!("  git-helper undo");
    println!("  git-helper undo 3");
    println!("  git-helper undo --hard");
    println!("  git-helper undo --count 5");
    println!("  git-helper undo -n 2 --hard");
}
