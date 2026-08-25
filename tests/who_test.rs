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
fn who_requires_cmd() {
    crux().arg("who").assert().failure();
}

#[test]
fn who_fails_outside_git_repo() {
    crux().arg("who").args(["-c", "cargo test"]).assert().failure();
}

#[test]
fn who_fails_no_change_in_range() {
    let tmp = TempDir::new().unwrap();
    make_git_repo(tmp.path());
    crux()
        .current_dir(tmp.path())
        .arg("who")
        .args(["-c", "echo ok", "-f", "HEAD~2..HEAD"])
        .assert()
        .failure();
}

fn sh_available() -> bool {
    std::process::Command::new("sh")
        .args(["-c", "true"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
}

fn make_flip_repo(dir: &std::path::Path) {
    Command::new("git").args(["init"]).current_dir(dir).assert().success();
    Command::new("git").args(["config", "user.name", "test"]).current_dir(dir).assert().success();
    Command::new("git").args(["config", "user.email", "test@test.com"]).current_dir(dir).assert().success();
    fs::write(dir.join("file.txt"), "pass\n").unwrap();
    Command::new("git").args(["add", "."]).current_dir(dir).assert().success();
    Command::new("git").args(["commit", "-m", "good"]).current_dir(dir).assert().success();
    fs::write(dir.join("file.txt"), "fail\n").unwrap();
    Command::new("git").args(["add", "."]).current_dir(dir).assert().success();
    Command::new("git").args(["commit", "-m", "bad"]).current_dir(dir).assert().success();
    fs::write(dir.join("file.txt"), "fail\nmore\n").unwrap();
    Command::new("git").args(["add", "."]).current_dir(dir).assert().success();
    Command::new("git").args(["commit", "-m", "also bad"]).current_dir(dir).assert().success();
}

#[test]
fn who_finds_flip_commit() {
    if !sh_available() { return; }
    let tmp = TempDir::new().unwrap();
    make_flip_repo(tmp.path());
    let output = crux()
        .current_dir(tmp.path())
        .arg("who")
        .args(["-c", "grep -q pass file.txt", "-f", "HEAD~2..HEAD"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stdout.contains("bad") || stderr.contains("bad"),
        "expected 'bad' in output: stdout={stdout} stderr={stderr}"
    );
}

#[test]
fn who_fast_mode_skips_suspects() {
    if !sh_available() { return; }
    let tmp = TempDir::new().unwrap();
    make_flip_repo(tmp.path());
    let output = crux()
        .current_dir(tmp.path())
        .arg("who")
        .args(["-c", "grep -q pass file.txt", "-f", "HEAD~2..HEAD", "--fast"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stdout.contains("bad") || stderr.contains("bad"),
        "expected 'bad' in output: stdout={stdout} stderr={stderr}"
    );
    assert!(
        !stderr.contains("suspects:"),
        "fast mode should skip suspects: {stderr}"
    );
}

#[test]
fn who_shows_ranked_scores() {
    if !sh_available() { return; }
    let tmp = TempDir::new().unwrap();
    Command::new("git").args(["init"]).current_dir(tmp.path()).assert().success();
    Command::new("git").args(["config", "user.name", "test"]).current_dir(tmp.path()).assert().success();
    Command::new("git").args(["config", "user.email", "test@test.com"]).current_dir(tmp.path()).assert().success();
    fs::write(tmp.path().join("file.txt"), "pass\n").unwrap();
    Command::new("git").args(["add", "."]).current_dir(tmp.path()).assert().success();
    Command::new("git").args(["commit", "-m", "base"]).current_dir(tmp.path()).assert().success();
    fs::write(tmp.path().join("file.txt"), "fail\n").unwrap();
    Command::new("git").args(["add", "."]).current_dir(tmp.path()).assert().success();
    Command::new("git").args(["commit", "-m", "break"]).current_dir(tmp.path()).assert().success();
    fs::write(tmp.path().join("file.txt"), "pass\nmore\n").unwrap();
    Command::new("git").args(["add", "."]).current_dir(tmp.path()).assert().success();
    Command::new("git").args(["commit", "-m", "fix"]).current_dir(tmp.path()).assert().success();
    let output = crux()
        .current_dir(tmp.path())
        .arg("who")
        .args(["-c", "grep -q pass file.txt", "-f", "HEAD~2..HEAD"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("ranked candidates"),
        "expected ranked candidates: {stderr}"
    );
}

fn make_interaction_repo(dir: &std::path::Path) {
    Command::new("git").args(["init"]).current_dir(dir).assert().success();
    Command::new("git").args(["config", "user.name", "test"]).current_dir(dir).assert().success();
    Command::new("git").args(["config", "user.email", "test@test.com"]).current_dir(dir).assert().success();
    fs::write(dir.join("file.txt"), "pass\n").unwrap();
    fs::write(dir.join("test.sh"), "#!/bin/sh\ngrep -q pass file.txt\n").unwrap();
    Command::new("git").args(["add", "."]).current_dir(dir).assert().success();
    Command::new("git").args(["commit", "-m", "base"]).current_dir(dir).assert().success();
    fs::write(dir.join("file.txt"), "pass\nextra\n").unwrap();
    Command::new("git").args(["add", "."]).current_dir(dir).assert().success();
    Command::new("git").args(["commit", "-m", "add extra"]).current_dir(dir).assert().success();
    fs::write(dir.join("test.sh"), "#!/bin/sh\ngrep -q pass file.txt && ! grep -q extra file.txt\n").unwrap();
    Command::new("git").args(["add", "."]).current_dir(dir).assert().success();
    Command::new("git").args(["commit", "-m", "change test"]).current_dir(dir).assert().success();
}

#[test]
fn who_finds_interaction_fault() {
    if !sh_available() { return; }
    let tmp = TempDir::new().unwrap();
    make_interaction_repo(tmp.path());
    let output = crux()
        .current_dir(tmp.path())
        .arg("who")
        .args(["-c", "sh test.sh", "-f", "HEAD~2..HEAD", "--interactions"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("interaction fault"), "expected interaction fault: {stderr}");
}

#[test]
fn who_interaction_alone_passes() {
    if !sh_available() { return; }
    let tmp = TempDir::new().unwrap();
    make_interaction_repo(tmp.path());
    let output = crux()
        .current_dir(tmp.path())
        .arg("who")
        .args(["-c", "sh test.sh", "-f", "HEAD~1..HEAD"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("change test"), "expected single blame: {stdout}");
}
