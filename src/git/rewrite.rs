use std::path::Path;

/// If `flip` looks like a squash/rebase/amend wrapper, hunt the original
/// commit by matching stable patch-ids across the reflog. Returns the
/// original hash, or None when no wrapper signature or match exists.
pub fn find_original(flip: &str, repo: &Path) -> Option<String> {
    let msg = commit_message(flip, repo);
    let reworded = ["squash", "rebase", "amend", "wip"]
        .iter()
        .any(|w| msg.to_lowercase().contains(w));
    if !reworded && !reflog_has_rewrite(repo) {
        return None;
    }

    let target = patch_id(flip, repo)?;
    for h in reflog_hashes(repo) {
        if h == flip {
            continue;
        }
        if let Some(pid) = patch_id(&h, repo)
            && pid == target
        {
            return Some(h);
        }
    }
    None
}

fn commit_message(hash: &str, repo: &Path) -> String {
    std::process::Command::new("git")
        .args(["log", "-1", "--format=%s", hash])
        .current_dir(repo)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

fn reflog_has_rewrite(repo: &Path) -> bool {
    let out = std::process::Command::new("git")
        .args(["reflog", "--format=%gs"])
        .current_dir(repo)
        .output();
    match out {
        Ok(o) => {
            let s = String::from_utf8_lossy(&o.stdout).to_lowercase();
            ["rebase", "commit (amend)", "reset: moving"].iter().any(|w| s.contains(w))
        }
        Err(_) => false,
    }
}

fn reflog_hashes(repo: &Path) -> Vec<String> {
    let out = std::process::Command::new("git")
        .args(["reflog", "--format=%H"])
        .current_dir(repo)
        .output();
    match out {
        Ok(o) => {
            let mut seen = std::collections::HashSet::new();
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .filter(|l| !l.is_empty())
                .filter(|h| seen.insert(h.to_string()))
                .take(200)
                .map(String::from)
                .collect()
        }
        Err(_) => Vec::new(),
    }
}

fn patch_id(hash: &str, repo: &Path) -> Option<String> {
    let show = std::process::Command::new("git")
        .args(["show", hash])
        .current_dir(repo)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn();
    let mut child = show.ok()?;
    let pid = std::process::Command::new("git")
        .args(["patch-id", "--stable"])
        .current_dir(repo)
        .stdin(child.stdout.take()?)
        .output()
        .ok()?;
    child.wait().ok()?;
    let line = String::from_utf8_lossy(&pid.stdout);
    line.lines().next().and_then(|l| l.split_whitespace().next()).map(String::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_rewrite_in_fresh_repo() {
        let dir = tempfile::TempDir::new().unwrap();
        let repo = dir.path();
        for args in [
            vec!["init"],
            vec!["-c", "user.name=t", "-c", "user.email=t@t", "commit", "--allow-empty", "-m", "a"],
        ] {
            std::process::Command::new("git")
                .args(&args)
                .current_dir(repo)
                .output()
                .unwrap();
        }
        let head = String::from_utf8_lossy(
            &std::process::Command::new("git")
                .args(["rev-parse", "HEAD"])
                .current_dir(repo)
                .output()
                .unwrap()
                .stdout,
        )
        .trim()
        .to_string();
        assert_eq!(find_original(&head, repo), None);
    }
}
