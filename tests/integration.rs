use std::fs;
use std::process::Command;

fn setup_test_repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path();

    Command::new("git")
        .args(&["init"])
        .current_dir(path)
        .output()
        .unwrap();

    Command::new("git")
        .args(&["config", "user.email", "test@test.com"])
        .current_dir(path)
        .output()
        .unwrap();

    Command::new("git")
        .args(&["config", "user.name", "Test User"])
        .current_dir(path)
        .output()
        .unwrap();

    Command::new("git")
        .args(&["config", "commit.gpgsign", "false"])
        .current_dir(path)
        .output()
        .unwrap();

    dir
}

fn git_helper(args: &[&str], repo_path: &std::path::Path) -> std::process::Output {
    let binary = if cfg!(target_os = "windows") {
        "target/debug/git-helper.exe"
    } else {
        "target/debug/git-helper"
    };

    Command::new(binary)
        .args(args)
        .current_dir(repo_path)
        .output()
        .unwrap()
}

fn make_initial_commit(path: &std::path::Path) {
    fs::write(path.join(".gitkeep"), "").unwrap();
    let out = git_helper(&["save", "initial commit"], path);
    assert!(out.status.success(), "initial commit failed: {:?}", String::from_utf8_lossy(&out.stderr));
}

#[test]
fn test_save_and_undo() {
    let dir = setup_test_repo();
    let path = dir.path();

    make_initial_commit(path);

    fs::write(path.join("test.txt"), "hello world").unwrap();
    let output = git_helper(&["save", "second commit"], path);
    assert!(output.status.success(), "save failed: {:?}", String::from_utf8_lossy(&output.stderr));

    let log = Command::new("git")
        .args(&["log", "--oneline"])
        .current_dir(path)
        .output()
        .unwrap();
    let log_out = String::from_utf8_lossy(&log.stdout);
    assert!(log_out.contains("second commit"), "commit not found in log");

    let output = git_helper(&["undo"], path);
    assert!(output.status.success(), "undo failed: {:?}", String::from_utf8_lossy(&output.stderr));

    let log = Command::new("git")
        .args(&["log", "--oneline"])
        .current_dir(path)
        .output()
        .unwrap();
    let log_out = String::from_utf8_lossy(&log.stdout);
    assert!(!log_out.contains("second commit"), "commit should have been undone");
    assert!(log_out.contains("initial commit"), "initial commit should remain");
}

#[test]
fn test_history() {
    let dir = setup_test_repo();
    let path = dir.path();

    make_initial_commit(path);

    fs::write(path.join("a.txt"), "a").unwrap();
    git_helper(&["save", "commit a"], path);

    fs::write(path.join("b.txt"), "b").unwrap();
    git_helper(&["save", "commit b"], path);

    let output = git_helper(&["history"], path);
    assert!(output.status.success(), "history failed: {:?}", String::from_utf8_lossy(&output.stderr));

    let output = git_helper(&["history", "--graph"], path);
    assert!(output.status.success(), "history --graph failed");
}

#[test]
fn test_status_no_repo() {
    let tmp = tempfile::tempdir().unwrap();
    let output = git_helper(&["status"], tmp.path());
    assert!(!output.status.success(), "status should fail outside repo");
}

#[test]
fn test_help() {
    let dir = setup_test_repo();
    let output = git_helper(&["help"], dir.path());
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("save"));
    assert!(stdout.contains("undo"));
    assert!(stdout.contains("sync"));
    assert!(stdout.contains("history"));
}

#[test]
fn test_stash() {
    let dir = setup_test_repo();
    let path = dir.path();

    make_initial_commit(path);

    fs::write(path.join("stash_test.txt"), "stash me").unwrap();
    let out = git_helper(&["save", "add tracked file"], path);
    assert!(out.status.success());

    fs::write(path.join("stash_test.txt"), "modified content").unwrap();
    let output = git_helper(&["stash"], path);
    assert!(output.status.success(), "stash failed: {:?}", String::from_utf8_lossy(&output.stderr));

    let output = git_helper(&["stash", "list"], path);
    assert!(output.status.success());

    let output = git_helper(&["stash", "pop"], path);
    assert!(output.status.success(), "stash pop failed: {:?}", String::from_utf8_lossy(&output.stderr));
}

#[test]
fn test_version() {
    let dir = setup_test_repo();
    let output = git_helper(&["version"], dir.path());
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("git-helper"));
}
