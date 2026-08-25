pub mod rewrite;

use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct Commit {
    pub hash: String,
    pub message: String,
    pub files_changed: Vec<String>,
}

pub fn log(range: &str, cwd: &Path) -> Result<Vec<Commit>, String> {
    log_inner(range, &[], false, cwd)
}

pub fn log_no_merges(range: &str, cwd: &Path) -> Result<Vec<Commit>, String> {
    log_inner(range, &[], true, cwd)
}

pub fn log_follow(file: &str, range: &str, cwd: &Path) -> Result<Vec<Commit>, String> {
    log_inner(range, &["--follow", "--", file], false, cwd)
}

fn log_inner(range: &str, extra: &[&str], no_merges: bool, cwd: &Path) -> Result<Vec<Commit>, String> {
    let mut args = vec!["log", "--format=%x1e%H %s", "--name-only", range];
    if no_merges {
        args.push("--no-merges");
    }
    args.extend(extra);
    let output = Command::new("git")
        .args(&args)
        .current_dir(cwd)
        .output()
        .map_err(|e| format!("git log: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git log failed: {stderr}"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut commits = Vec::new();
    for chunk in stdout.split('\u{1e}') {
        let mut lines = chunk.lines().filter(|l| !l.is_empty());
        let Some(head) = lines.next() else { continue };
        let (hash, message) = head.split_once(' ').unwrap_or((head, ""));
        commits.push(Commit {
            hash: hash.to_string(),
            message: message.to_string(),
            files_changed: lines.map(String::from).collect(),
        });
    }
    Ok(commits)
}

/// Range covering all reachable commits, safe for shallow/short histories.
pub fn full_range(cwd: &Path) -> String {
    let out = Command::new("git")
        .args(["rev-list", "--max-parents=0", "HEAD"])
        .current_dir(cwd)
        .output();
    let root = out
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    match root.lines().last() {
        Some(r) if !r.is_empty() => format!("{r}..HEAD"),
        _ => "HEAD".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_populates_files() {
        let dir = tempfile::TempDir::new().unwrap();
        let repo = dir.path();
        Command::new("git").args(["init"]).current_dir(repo).output().unwrap();
        Command::new("git").args(["-c", "user.name=test", "-c", "user.email=t@t.t", "commit", "--allow-empty", "-m", "first"]).current_dir(repo).output().unwrap();
        std::fs::write(repo.join("a.txt"), "hello").unwrap();
        Command::new("git").args(["-c", "user.name=test", "-c", "user.email=t@t.t", "add", "."]).current_dir(repo).output().unwrap();
        Command::new("git").args(["-c", "user.name=test", "-c", "user.email=t@t.t", "commit", "-m", "second"]).current_dir(repo).output().unwrap();
        let commits = log("HEAD~1..HEAD", repo).unwrap();
        assert_eq!(commits.len(), 1);
        assert!(commits[0].files_changed.contains(&"a.txt".to_string()));
    }
}
