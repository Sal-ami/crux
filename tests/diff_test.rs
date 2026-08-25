use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

fn crux() -> Command {
    let mut cmd = Command::cargo_bin("crux").unwrap();
    cmd.current_dir(env!("CARGO_MANIFEST_DIR"));
    cmd
}

fn make_git_repo(dir: &std::path::Path) {
    Command::new("git").args(["init"]).current_dir(dir).assert().success();
    Command::new("git").args(["config", "user.name", "test"]).current_dir(dir).assert().success();
    Command::new("git").args(["config", "user.email", "test@test.com"]).current_dir(dir).assert().success();
    fs::write(dir.join("file.txt"), "a\n").unwrap();
    Command::new("git").args(["add", "."]).current_dir(dir).assert().success();
    Command::new("git").args(["commit", "-m", "first"]).current_dir(dir).assert().success();
    fs::write(dir.join("file.txt"), "a\nb\n").unwrap();
    Command::new("git").args(["add", "."]).current_dir(dir).assert().success();
    Command::new("git").args(["commit", "-m", "second"]).current_dir(dir).assert().success();
    fs::write(dir.join("file.txt"), "a\nb\nc\n").unwrap();
    Command::new("git").args(["add", "."]).current_dir(dir).assert().success();
    Command::new("git").args(["commit", "-m", "third"]).current_dir(dir).assert().success();
}

#[test]
fn diff_requires_range() {
    crux().arg("diff").assert().failure();
}

#[test]
fn diff_shows_commits() {
    let tmp = TempDir::new().unwrap();
    make_git_repo(tmp.path());
    let output = crux()
        .current_dir(tmp.path())
        .arg("diff")
        .arg("HEAD~2..HEAD")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("second") || stdout.contains("third"),
        "unexpected output: {stdout}"
    );
}

#[test]
fn diff_json_output() {
    let tmp = TempDir::new().unwrap();
    make_git_repo(tmp.path());
    let output = crux()
        .current_dir(tmp.path())
        .arg("diff")
        .args(["HEAD~2..HEAD", "-o", "json"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("["), "expected JSON array: {stdout}");
}
