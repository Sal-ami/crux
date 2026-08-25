pub mod dep;
pub mod interaction;
pub mod parallel;
pub mod upstream;

use std::path::Path;
use std::process::Command;

use crate::git;

#[derive(Debug, Clone)]
pub struct Blame {
    pub commit: git::Commit,
    pub confidence: f64,
}

fn checkout_detached(hash: &str, repo: &Path) -> bool {
    Command::new("git")
        .args(["checkout", "-f", "--quiet", hash])
        .current_dir(repo)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn run_at(hash: &str, cmd: &str, repo: &Path) -> bool {
    if !checkout_detached(hash, repo) {
        return false;
    }
    let (shell, args) = crate::shell_cmd();
    let status = Command::new(shell)
        .args(args)
        .arg(cmd)
        .current_dir(repo)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    matches!(status, Ok(s) if s.success())
}

/// Run the behavior at `hash`, capturing combined stdout+stderr.
pub fn run_at_output(hash: &str, cmd: &str, repo: &Path) -> String {
    if !checkout_detached(hash, repo) {
        return String::new();
    }
    let (shell, args) = crate::shell_cmd();
    match Command::new(shell)
        .args(args)
        .arg(cmd)
        .current_dir(repo)
        .output()
    {
        Ok(o) => {
            let mut s = String::from_utf8_lossy(&o.stdout).into_owned();
            s.push_str(&String::from_utf8_lossy(&o.stderr));
            s
        }
        Err(_) => String::new(),
    }
}

pub fn restore_head(head: &str, repo: &Path) {
    let _ = checkout_detached(head, repo);
}



pub fn blame(cmd: &str, range: &str, no_merges: bool, repo: &Path) -> Result<Blame, String> {
    let mut all_commits = if no_merges {
        git::log_no_merges(range, repo)?
    } else {
        git::log(range, repo)?
    };
    if all_commits.is_empty() {
        return Err("no commits in range".into());
    }
    all_commits.reverse();
    if all_commits.len() == 1 {
        return Ok(Blame {
            commit: all_commits.into_iter().next().unwrap(),
            confidence: 1.0,
        });
    }

    let orig = Command::new("git")
        .args(["rev-parse", "--quiet", "HEAD"])
        .current_dir(repo)
        .output()
        .map_err(|e| format!("git rev-parse: {e}"))?;
    let orig_head = String::from_utf8_lossy(&orig.stdout).trim().to_string();

    let last_passes = run_at(&all_commits.last().unwrap().hash, cmd, repo);

    let result = if !last_passes {
        // Probe only the tip; assume the root passes. If the window narrows to
        // index 0 anyway, verify the root explicitly (whole-range breakage).
        let mut lo = 0usize;
        let mut hi = all_commits.len() - 1;
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            if run_at(&all_commits[mid].hash, cmd, repo) {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        if lo == 0 && run_at(&all_commits[0].hash, cmd, repo) {
            None
        } else {
            Some(lo)
        }
    } else {
        None
    };

    restore_head(&orig_head, repo);

    match result {
        Some(i) => Ok(Blame {
            commit: all_commits.into_iter().nth(i).unwrap(),
            confidence: 1.0,
        }),
        None => Err("no behavior change detected in range".into()),
    }
}

#[derive(Debug, Clone)]
pub struct Ranked {
    pub commit: git::Commit,
    pub score: f64,
}

/// Evidence-ranked candidates when the history is not monotone:
/// gradual drift or multiple flip points. O(n) test executions.
pub fn ranked(cmd: &str, range: &str, no_merges: bool, repo: &Path) -> Result<Vec<Ranked>, String> {
    let orig_head = Command::new("git")
        .args(["rev-parse", "--quiet", "HEAD"])
        .current_dir(repo)
        .output()
        .map_err(|e| format!("git rev-parse: {e}"))?;
    let orig = String::from_utf8_lossy(&orig_head.stdout).trim().to_string();

    let out = ranked_inner(cmd, range, no_merges, repo);
    restore_head(&orig, repo);
    out
}

fn ranked_inner(
    cmd: &str,
    range: &str,
    no_merges: bool,
    repo: &Path,
) -> Result<Vec<Ranked>, String> {
    let mut commits = if no_merges {
        git::log_no_merges(range, repo)?
    } else {
        git::log(range, repo)?
    };
    if commits.is_empty() {
        return Err("no commits in range".into());
    }
    commits.reverse();

    let results: Vec<bool> = commits
        .iter()
        .map(|c| run_at(&c.hash, cmd, repo))
        .collect();
    let flips: usize = results.windows(2).filter(|w| w[0] != w[1]).count();
    let failing: usize = results.iter().filter(|p| !**p).count();
    if flips == 0 {
        return Err(if failing == commits.len() {
            "all commits fail in range".into()
        } else {
            "no behavior change detected in range".into()
        });
    }

    let n = commits.len();
    let mut scored: Vec<Ranked> = commits
        .iter()
        .zip(results.iter())
        .enumerate()
        .filter(|(_, (_, passed))| !**passed)
        .map(|(i, (c, _))| {
            let boundary = i + 1 == n || results[i + 1];
            let mut score = 0.0;
            if boundary {
                score += 50.0;
            }
            score += 30.0 * ((n - i) as f64 / n as f64);
            if c.files_changed.len() <= 5 {
                score += 20.0;
            }
            Ranked {
                commit: c.clone(),
                score,
            }
        })
        .collect();
    scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(5);
    Ok(scored)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn commit_all(repo: &Path, msg: &str) {
        Command::new("git")
            .args(["-c", "user.name=t", "-c", "user.email=t@t", "commit", "--allow-empty", "-q", "-m", msg])
            .current_dir(repo)
            .output()
            .unwrap();
    }

    #[test]
    fn ranked_restores_head_after_scan() {
        let dir = tempfile::TempDir::new().unwrap();
        let repo = dir.path();
        Command::new("git").args(["init"]).current_dir(repo).output().unwrap();
        std::fs::write(repo.join("f.txt"), "pass").unwrap();
        for i in 1..=4 {
            if i == 3 {
                std::fs::write(repo.join("f.txt"), "fail").unwrap();
            }
            let mut add = Command::new("git");
            add.args(["add", "."]).current_dir(repo).output().unwrap();
            let _ = i;
            commit_all(repo, &format!("c{i}"));
        }
        // make it non-monotone so binary search fails and ranked runs:
        // flip back to pass at the end
        std::fs::write(repo.join("f.txt"), "pass").unwrap();
        Command::new("git").args(["add", "."]).current_dir(repo).output().unwrap();
        commit_all(repo, "c5");

        let before = String::from_utf8_lossy(
            &Command::new("git")
                .args(["rev-parse", "HEAD"])
                .current_dir(repo)
                .output()
                .unwrap()
                .stdout,
        )
        .trim()
        .to_string();

        // predicate: file content contains "fail"
        let cmd = if cfg!(windows) {
            "findstr /i \"fail\" f.txt >nul 2>&1 && (exit /b 1) || (exit /b 0)"
        } else {
            "grep -qi fail f.txt && exit 1 || exit 0"
        };
        let root = String::from_utf8_lossy(
            &Command::new("git")
                .args(["rev-list", "--max-parents=0", "HEAD"])
                .current_dir(repo)
                .output()
                .unwrap()
                .stdout,
        )
        .trim()
        .to_string();
        let range = format!("{root}..HEAD");
        // monotone here -> ranked errors, but the scan still checked out every commit
        let _ = ranked(cmd, &range, false, repo);

        let after = String::from_utf8_lossy(
            &Command::new("git")
                .args(["rev-parse", "HEAD"])
                .current_dir(repo)
                .output()
                .unwrap()
                .stdout,
        )
        .trim()
        .to_string();
        assert_eq!(before, after, "ranked must restore HEAD");
    }
}
