use std::path::Path;

use crate::git::Commit;

pub struct Interaction {
    pub participants: Vec<Commit>,
    pub probes: usize,
}

/// Find the minimal set of earlier commits that must be present alongside
/// `flip` for the behavior to be broken. Returns None when flip alone
/// explains the failure (no interaction).
pub fn detect(
    cmd: &str,
    flip: &Commit,
    earlier: &[Commit],
    repo: &Path,
) -> Option<Interaction> {
    let candidates: Vec<&Commit> = earlier.iter().filter(|c| c.hash != flip.hash).take(16).collect();
    if candidates.is_empty() {
        return None;
    }

    let mut probes = 0usize;
    let mut broken_with = |present: &[bool]| -> bool {
        probes += 1;
        checkout(&flip.hash, repo);
        for (i, c) in candidates.iter().enumerate() {
            if !present[i] && !revert(c, repo) {
                // cannot revert cleanly -> treat as forced-present
                return true;
            }
        }
        // test the CURRENT worktree; run_at would discard our reverts
        crate::sandbox::local::run_in_sandbox(cmd, repo).is_none()
    };

    let full: Vec<bool> = vec![true; candidates.len()];
    if !broken_with(&full) {
        return None; // flip alone explains it
    }
    let kept = reduce(candidates.len(), &mut broken_with);
    let mut participants = vec![flip.clone()];
    for (i, c) in candidates.iter().enumerate() {
        if kept[i] {
            participants.push((*c).clone());
        }
    }
    Some(Interaction { participants, probes })
}

fn reduce(n: usize, broken_with: &mut impl FnMut(&[bool]) -> bool) -> Vec<bool> {
    let mut present: Vec<bool> = vec![true; n];
    let mut changed = true;
    while changed {
        changed = false;
        for i in 0..n {
            if !present[i] {
                continue;
            }
            present[i] = false;
            if !broken_with(&present) {
                present[i] = true; // required participant
            } else {
                changed = true;
            }
        }
    }
    present
}

fn revert(c: &Commit, repo: &Path) -> bool {
    let out = std::process::Command::new("git")
        .args(["diff", "--no-color", &format!("{}^", c.hash), &c.hash])
        .current_dir(repo)
        .output();
    let diff = match out {
        Ok(o) if o.status.success() => o.stdout,
        _ => return false,
    };
    let apply = std::process::Command::new("git")
        .args(["apply", "-R", "--whitespace=nowarn"])
        .current_dir(repo)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
    match apply {
        Ok(mut child) => {
            use std::io::Write;
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(&diff);
            }
            child.wait().map(|s| s.success()).unwrap_or(false)
        }
        Err(_) => false,
    }
}

fn checkout(hash: &str, repo: &Path) {
    let script = if cfg!(windows) {
        format!("git checkout -f --quiet {hash} 2>NUL")
    } else {
        format!("git checkout -f --quiet {hash} 2>/dev/null")
    };
    let (shell, args) = crate::shell_cmd();
    let _ = std::process::Command::new(shell)
        .args(args)
        .arg(&script)
        .current_dir(repo)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
}
