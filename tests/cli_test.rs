use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn crux() -> Command {
    let mut cmd = Command::cargo_bin("crux").unwrap();
    cmd.current_dir(env!("CARGO_MANIFEST_DIR"));
    cmd
}

#[test]
fn help_exits_zero() {
    crux().arg("--help").assert().success();
}

#[test]
fn help_contains_usage() {
    crux()
        .arg("--help")
        .assert()
        .stdout(predicate::str::contains("Usage:"));
}

#[test]
fn help_contains_all_subcommands() {
    let output = crux().arg("--help").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    for cmd in &["who", "diff", "init", "log", "replay", "doctor", "report", "index", "watch", "predict"] {
        assert!(stdout.contains(cmd), "missing subcommand: {cmd}");
    }
}

#[test]
fn who_requires_cmd() {
    crux()
        .arg("who")
        .assert()
        .failure();
}

#[test]
fn who_fails_outside_git_repo() {
    crux()
        .arg("who")
        .args(["-c", "cargo test"])
        .assert()
        .failure();
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

#[test]
fn diff_requires_range() {
    crux()
        .arg("diff")
        .assert()
        .failure();
}

#[test]
fn diff_dispatches() {
    let tmp = TempDir::new().unwrap();
    make_git_repo(tmp.path());
    let output = crux()
        .current_dir(tmp.path())
        .arg("diff")
        .arg("HEAD~2..HEAD")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stdout.contains("second") || stdout.contains("third") || stderr.contains("no changes"),
        "unexpected output: stdout={stdout} stderr={stderr}"
    );
}

#[test]
fn init_dispatches() {
    let tmp = TempDir::new().unwrap();
    make_git_repo(tmp.path());
    crux()
        .current_dir(tmp.path())
        .arg("init")
        .assert()
        .stderr(predicate::str::contains("crux init"));
}

#[test]
fn replay_requires_fingerprint() {
    crux()
        .arg("replay")
        .assert()
        .failure();
}

#[test]
fn replay_dispatches() {
    crux()
        .arg("replay")
        .arg("abc123")
        .assert()
        .stderr(predicate::str::contains("no signature found"));
}

#[test]
fn doctor_dispatches() {
    let tmp = TempDir::new().unwrap();
    make_git_repo(tmp.path());
    let output = crux()
        .current_dir(tmp.path())
        .arg("doctor")
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("checks passed") || stderr.contains("check"),
        "unexpected stderr: {stderr}"
    );
}

#[test]
fn report_requires_target() {
    crux()
        .arg("report")
        .assert()
        .failure();
}

#[test]
fn report_dispatches() {
    crux()
        .arg("report")
        .arg("run-42")
        .assert()
        .stderr(predicate::str::contains("no signature found for: run-42"));
}

#[test]
fn index_list_dispatches() {
    let tmp = TempDir::new().unwrap();
    crux()
        .current_dir(tmp.path())
        .args(["index", "list"])
        .assert()
        .stderr(predicate::str::contains("no signatures"));
}

#[test]
fn index_export_dispatches() {
    let tmp = TempDir::new().unwrap();
    let output = crux()
        .current_dir(tmp.path())
        .args(["index", "export"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn unknown_subcommand_fails() {
    crux()
        .arg("bogus")
        .assert()
        .failure();
}

fn make_git_repo(dir: &std::path::Path) {
    Command::new("git")
        .args(["init"])
        .current_dir(dir)
        .assert()
        .success();
    Command::new("git")
        .args(["config", "user.name", "test"])
        .current_dir(dir)
        .assert()
        .success();
    Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(dir)
        .assert()
        .success();
    fs::write(dir.join("file.txt"), "a\n").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(dir)
        .assert()
        .success();
    Command::new("git")
        .args(["commit", "-m", "first"])
        .current_dir(dir)
        .assert()
        .success();
    fs::write(dir.join("file.txt"), "a\nb\n").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(dir)
        .assert()
        .success();
    Command::new("git")
        .args(["commit", "-m", "second"])
        .current_dir(dir)
        .assert()
        .success();
    fs::write(dir.join("file.txt"), "a\nb\nc\n").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(dir)
        .assert()
        .success();
    Command::new("git")
        .args(["commit", "-m", "third"])
        .current_dir(dir)
        .assert()
        .success();
}

#[test]
fn log_requires_range() {
    crux()
        .arg("log")
        .assert()
        .failure();
}

#[test]
fn log_lists_commits() {
    let tmp = TempDir::new().unwrap();
    make_git_repo(tmp.path());
    let output = crux()
        .current_dir(tmp.path())
        .arg("log")
        .arg("HEAD~2..HEAD")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("third"), "missing third: {stdout}");
    assert!(stdout.contains("second"), "missing second: {stdout}");
    assert!(!stdout.contains("first"), "should not contain first: {stdout}");
}

#[test]
fn log_single_commit() {
    let tmp = TempDir::new().unwrap();
    make_git_repo(tmp.path());
    let output = crux()
        .current_dir(tmp.path())
        .arg("log")
        .arg("HEAD~1..HEAD")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("third"), "missing third: {stdout}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("1 commits"), "wrong count: {stderr}");
}

#[test]
fn log_not_a_repo() {
    let tmp = TempDir::new().unwrap();
    crux()
        .current_dir(tmp.path())
        .arg("log")
        .arg("HEAD")
        .assert()
        .failure();
}

fn make_interaction_repo(dir: &std::path::Path) {
    Command::new("git")
        .args(["init"])
        .current_dir(dir)
        .assert()
        .success();
    Command::new("git")
        .args(["config", "user.name", "test"])
        .current_dir(dir)
        .assert()
        .success();
    Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(dir)
        .assert()
        .success();
    fs::write(dir.join("file.txt"), "pass\n").unwrap();
    fs::write(dir.join("test.sh"), "#!/bin/sh\ngrep -q pass file.txt\n").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(dir)
        .assert()
        .success();
    Command::new("git")
        .args(["commit", "-m", "base"])
        .current_dir(dir)
        .assert()
        .success();
    fs::write(dir.join("file.txt"), "pass\nextra\n").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(dir)
        .assert()
        .success();
    Command::new("git")
        .args(["commit", "-m", "add extra"])
        .current_dir(dir)
        .assert()
        .success();
    fs::write(
        dir.join("test.sh"),
        "#!/bin/sh\ngrep -q pass file.txt && ! grep -q extra file.txt\n",
    )
    .unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(dir)
        .assert()
        .success();
    Command::new("git")
        .args(["commit", "-m", "change test"])
        .current_dir(dir)
        .assert()
        .success();
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
    Command::new("git")
        .args(["init"])
        .current_dir(dir)
        .assert()
        .success();
    Command::new("git")
        .args(["config", "user.name", "test"])
        .current_dir(dir)
        .assert()
        .success();
    Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(dir)
        .assert()
        .success();
    fs::write(dir.join("file.txt"), "pass\n").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(dir)
        .assert()
        .success();
    Command::new("git")
        .args(["commit", "-m", "good"])
        .current_dir(dir)
        .assert()
        .success();
    fs::write(dir.join("file.txt"), "fail\n").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(dir)
        .assert()
        .success();
    Command::new("git")
        .args(["commit", "-m", "bad"])
        .current_dir(dir)
        .assert()
        .success();
    fs::write(dir.join("file.txt"), "pass\nmore\n").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(dir)
        .assert()
        .success();
    Command::new("git")
        .args(["commit", "-m", "fixed"])
        .current_dir(dir)
        .assert()
        .success();
}

#[test]
fn ranked_shows_flip_commit() {
    if !sh_available() {
        return;
    }
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
fn ranked_scores_all_commits() {
    if !sh_available() {
        return;
    }
    let tmp = TempDir::new().unwrap();
    make_flip_repo(tmp.path());
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

#[test]
fn interaction_finds_fault_pair() {
    if !sh_available() {
        return;
    }
    let tmp = TempDir::new().unwrap();
    make_interaction_repo(tmp.path());
    let output = crux()
        .current_dir(tmp.path())
        .arg("who")
        .args(["-c", "sh test.sh", "-f", "HEAD~2..HEAD", "--interactions"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("interaction fault"),
        "expected interaction fault: {stderr}"
    );
}

#[test]
fn interaction_no_fault_when_alone_passes() {
    if !sh_available() {
        return;
    }
    let tmp = TempDir::new().unwrap();
    make_interaction_repo(tmp.path());
    let output = crux()
        .current_dir(tmp.path())
        .arg("who")
        .args(["-c", "sh test.sh", "-f", "HEAD~1..HEAD"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("change test"),
        "expected single blame: {stdout}"
    );
}

#[test]
fn watch_requires_cmd() {
    crux().arg("watch").assert().failure();
}

#[test]
fn watch_stores_baseline() {
    let tmp = TempDir::new().unwrap();
    make_git_repo(tmp.path());
    let out = crux()
        .current_dir(tmp.path())
        .args(["watch", "-c", "echo ok"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("baseline stored"), "expected baseline: {stderr}");
}

#[test]
fn watch_detects_stable() {
    let tmp = TempDir::new().unwrap();
    make_git_repo(tmp.path());
    crux().current_dir(tmp.path()).args(["watch", "-c", "echo ok"]).assert().success();
    let out = crux()
        .current_dir(tmp.path())
        .args(["watch", "-c", "echo ok"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("stable"), "expected stable: {stderr}");
}

#[test]
fn predict_no_change() {
    let tmp = TempDir::new().unwrap();
    make_git_repo(tmp.path());
    let out = crux()
        .current_dir(tmp.path())
        .args(["predict", "-c", "echo ok", "-f", "HEAD~2..HEAD"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("no behavior change"), "expected no change: {stderr}");
}

#[test]
fn who_follow_flag() {
    let tmp = TempDir::new().unwrap();
    make_git_repo(tmp.path());
    crux()
        .current_dir(tmp.path())
        .args(["who", "-c", "echo ok", "-f", "HEAD~2..HEAD", "--follow"])
        .assert()
        .failure();
}
