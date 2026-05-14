pub mod history;
pub mod save;
pub mod sync;
pub mod undo;

use colored::Colorize;

pub fn print_help() {
    println!("{}", "Git Helper CLI — Simplify your Git workflow".bold());
    println!("{}", "=".repeat(50));
    println!();
    println!("{}", "USAGE:");
    println!("  git-helper <command> [options] [args]");
    println!();
    println!("{}", "COMMANDS:");
    println!("  {}     Stage all, commit, and push in one step", "save".cyan());
    println!("           git-helper save \"commit message\"");
    println!("           git-helper save --skip-push \"wip\"");
    println!("           git-helper save --amend \"better message\"");
    println!();
    println!("  {}     Undo the last commit (keep changes staged)", "undo".cyan());
    println!("           git-helper undo");
    println!("           git-helper undo --hard");
    println!("           git-helper undo -n 3");
    println!();
    println!("  {}     Pull and push in one command", "sync".cyan());
    println!("           git-helper sync");
    println!("           git-helper sync --rebase");
    println!();
    println!("  {}     View clean commit history", "history".cyan());
    println!("           git-helper history");
    println!("           git-helper history --graph --all");
    println!();
    println!("  {}     Show current repo status", "status".cyan());
    println!("           git-helper status");
    println!();
    println!("  {}     Stash/unstash changes", "stash".cyan());
    println!("           git-helper stash");
    println!("           git-helper stash pop");
    println!();
    println!("  {}     Show this help message", "help".cyan());
    println!();
    println!("{}", "EXAMPLES:");
    println!("  git-helper save \"feat: add user authentication\"");
    println!("  git-helper undo");
    println!("  git-helper sync --rebase");
    println!("  git-helper history --graph");
    println!("  git-helper status");
}
