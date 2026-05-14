use crate::utils::git;

pub fn execute(args: &[String]) {
    if !args.is_empty() && (args[0] == "--help" || args[0] == "-h") {
        print_usage();
        return;
    }

    git::require_git_repo();

    let mut count: usize = 20;
    let mut show_graph = false;
    let mut all_branches = false;
    let mut oneline = true;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--graph" | "-g" => {
                show_graph = true;
            }
            "--all" | "-a" => {
                all_branches = true;
            }
            "--oneline" | "-o" => {
                oneline = true;
            }
            "--detailed" | "-d" => {
                oneline = false;
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
            "--help" | "-h" => {
                print_usage();
                return;
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
        count = usize::MAX;
    }

    let mut git_args: Vec<String> = vec!["log".to_string()];

    if show_graph {
        git_args.push("--graph".to_string());
        git_args.push("--pretty=format:%C(yellow)%h%Creset %C(cyan)%an%Creset %s %C(green)(%ar)%Creset".to_string());
    } else if oneline {
        git_args.push("--oneline".to_string());
    } else {
        git_args.push("--pretty=format:%C(yellow)%h%Creset %C(cyan)%an%Creset <%C(blue)%ae%Creset>".to_string());
        git_args.push("-p".to_string());
    }

    if all_branches {
        git_args.push("--all".to_string());
    }

    if count != usize::MAX {
        git_args.push(format!("-{}", count));
    }

    let refs: Vec<&str> = git_args.iter().map(|s| s.as_str()).collect();

    let branch = git::get_current_branch();
    println!("Commit history for branch: {} {}", branch, if all_branches { "(all branches)" } else { "" });
    println!("{}", "-".repeat(50));

    match git::run(&refs) {
        git::GitResult::Failed(e) => {
            eprintln!("error: failed to get history: {}", e);
            std::process::exit(1);
        }
        _ => {}
    }
}

fn print_usage() {
    println!("Usage: git-helper history [options]");
    println!();
    println!("View a clean commit history.");
    println!();
    println!("Options:");
    println!("  --graph, -g     Show graph view with branches");
    println!("  --all, -a       Show commits from all branches");
    println!("  --oneline, -o   Compact oneline format (default)");
    println!("  --detailed, -d  Show full commit details");
    println!("  --count <N>, -n <N>  Limit to N commits");
    println!("  --help, -h      Show this help message");
    println!();
    println!("Examples:");
    println!("  git-helper history");
    println!("  git-helper history --graph");
    println!("  git-helper history -g -a");
    println!("  git-helper history -n 5");
    println!("  git-helper history --detailed -n 3");
}
