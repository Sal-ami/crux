use std::path::Path;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::git;

#[derive(Debug)]
pub struct ParallelResult {
    pub commit: git::Commit,
    pub passed: bool,
}

pub fn parallel_bisect(
    cmd: &str,
    range: &str,
    repo: &Path,
    workers: usize,
) -> Result<Vec<ParallelResult>, String> {
    let commits = git::log(range, repo)?;
    if commits.is_empty() {
        return Ok(vec![]);
    }
    let mut commits = commits;
    commits.reverse();
    if commits.len() == 1 {
        let passed = crate::blame::run_at(&commits[0].hash, cmd, repo);
        return Ok(vec![ParallelResult { commit: commits[0].clone(), passed }]);
    }

    let worktree_base = repo.join(".crux").join("worktrees");
    std::fs::create_dir_all(&worktree_base).map_err(|e| format!("create worktrees: {e}"))?;

    let results: Arc<Mutex<Vec<ParallelResult>>> = Arc::new(Mutex::new(Vec::new()));
    let commits = Arc::new(commits);
    let cmd = Arc::new(cmd.to_string());
    let repo = Arc::new(repo.to_path_buf());
    let worktree_base = Arc::new(worktree_base);

    let chunk_size = commits.len().div_ceil(workers);
    let mut handles = Vec::new();

    for chunk_idx in 0..workers {
        let start = chunk_idx * chunk_size;
        let end = (start + chunk_size).min(commits.len());
        if start >= commits.len() {
            break;
        }

        let results = Arc::clone(&results);
        let commits = Arc::clone(&commits);
        let cmd = Arc::clone(&cmd);
        let repo = Arc::clone(&repo);
        let worktree_base = Arc::clone(&worktree_base);

        handles.push(thread::spawn(move || {
            for i in start..end {
                let commit = &commits[i];
                let wt_name = format!("wt_{chunk_idx}_{i}");
                let wt_path = worktree_base.join(&wt_name);

                let _ = remove_worktree(&wt_path, &repo);
                if create_worktree(&wt_path, &commit.hash, &repo).is_err() {
                    let passed = crate::blame::run_at(&commit.hash, &cmd, &repo);
                    if let Ok(mut r) = results.lock() {
                        r.push(ParallelResult {
                            commit: commit.clone(),
                            passed,
                        });
                    }
                    continue;
                }

                let passed = run_in_worktree(&cmd, &wt_path);
                let _ = remove_worktree(&wt_path, &repo);

                if let Ok(mut r) = results.lock() {
                    r.push(ParallelResult {
                        commit: commit.clone(),
                        passed,
                    });
                }
            }
        }));
    }

    for h in handles {
        let _ = h.join();
    }

    let _ = std::fs::remove_dir_all(&*worktree_base);

    let results = Arc::try_unwrap(results)
        .map_err(|_| "threads still running".to_string())?
        .into_inner()
        .map_err(|_| "mutex poisoned".to_string())?;
    let commits = Arc::try_unwrap(commits)
        .map_err(|_| "threads still running".to_string())?;
    let mut results = results;
    results.sort_by_key(|r| {
        commits.iter().position(|c| c.hash == r.commit.hash).unwrap_or(0)
    });
    Ok(results)
}

fn create_worktree(path: &Path, hash: &str, repo: &Path) -> Result<(), String> {
    let path_str = path.to_str().ok_or("non-UTF-8 worktree path")?;
    let output = Command::new("git")
        .args(["worktree", "add", "--detach", path_str, hash])
        .current_dir(repo)
        .output()
        .map_err(|e| format!("git worktree add: {e}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).into_owned())
    }
}

fn remove_worktree(path: &Path, repo: &Path) -> Result<(), String> {
    let path_str = path.to_str().unwrap_or("");
    let _ = Command::new("git")
        .args(["worktree", "remove", "--force", path_str])
        .current_dir(repo)
        .output();
    let _ = std::fs::remove_dir_all(path);
    Ok(())
}

fn run_in_worktree(cmd: &str, worktree: &Path) -> bool {
    let (shell, args) = crate::shell_cmd();
    matches!(
        Command::new(shell)
            .args(args)
            .arg(cmd)
            .current_dir(worktree)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status(),
        Ok(s) if s.success()
    )
}

pub fn find_flip(results: &[ParallelResult]) -> Option<&ParallelResult> {
    results.windows(2).find_map(|w| {
        if w[0].passed && !w[1].passed {
            Some(&w[1])
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_flip_finds_change() {
        let results = vec![
            ParallelResult { commit: git::Commit { hash: "a".into(), message: "a".into(), files_changed: vec![] }, passed: true },
            ParallelResult { commit: git::Commit { hash: "b".into(), message: "b".into(), files_changed: vec![] }, passed: true },
            ParallelResult { commit: git::Commit { hash: "c".into(), message: "c".into(), files_changed: vec![] }, passed: false },
        ];
        let flip = find_flip(&results).unwrap();
        assert_eq!(flip.commit.hash, "c");
    }

    #[test]
    fn find_flip_none_when_all_pass() {
        let results = vec![
            ParallelResult { commit: git::Commit { hash: "a".into(), message: "a".into(), files_changed: vec![] }, passed: true },
            ParallelResult { commit: git::Commit { hash: "b".into(), message: "b".into(), files_changed: vec![] }, passed: true },
        ];
        assert!(find_flip(&results).is_none());
    }
}
