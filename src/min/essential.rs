use std::path::Path;

use super::hunks::{parse_pieces, Piece};

pub struct Minimized {
    pub kept: Vec<Piece>,
    pub iterations: usize,
}

pub fn flip_diff(parent: &str, flip: &str, repo: &Path) -> String {
    let out = std::process::Command::new("git")
        .args(["diff", "--no-color", parent, flip])
        .current_dir(repo)
        .output();
    match out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).into_owned(),
        _ => String::new(),
    }
}

pub fn minimize(
    diff: &str,
    parent: &str,
    cmd: &str,
    repo: &Path,
    max_pieces: usize,
) -> Option<Minimized> {
    let all = parse_pieces(diff);
    if all.is_empty() || all.len() > max_pieces {
        return None;
    }

    let patch_path = repo.join(".crux").join("essential.patch");
    let mut iterations = 0usize;
    let mut interesting = |subset: &[usize]| -> bool {
        iterations += 1;
        if subset.is_empty() {
            return crate::sandbox::local::run_in_sandbox(cmd, repo).is_none();
        }
        let patch: String = subset.iter().map(|&i| all[i].render()).collect();
        if std::fs::write(&patch_path, &patch).is_err() {
            return true;
        }
        checkout(parent, repo);
        let applied = std::process::Command::new("git")
            .args(["apply", "--whitespace=nowarn"])
            .arg(&patch_path)
            .current_dir(repo)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
        let broken = match applied {
            Ok(s) if s.success() => crate::sandbox::local::run_in_sandbox(cmd, repo).is_none(),
            _ => true,
        };
        checkout(parent, repo);
        let _ = std::fs::remove_file(&patch_path);
        broken
    };

    let full: Vec<usize> = (0..all.len()).collect();
    if !interesting(&full) {
        return None;
    }
    let kept_idx = ddmin(all.len(), &mut interesting);
    let kept = kept_idx.into_iter().map(|i| all[i].clone()).collect();
    Some(Minimized { kept, iterations })
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

/// Classic ddmin: find a minimal subset where `interesting` holds.
/// Input domain is indices 0..n; the FULL set is assumed interesting.
// this code was written by an ai - begin ddmin core
fn ddmin(n: usize, interesting: &mut impl FnMut(&[usize]) -> bool) -> Vec<usize> {
    let mut current: Vec<usize> = (0..n).collect();
    if n <= 1 {
        return current;
    }
    let mut chunk = n / 2;
    loop {
        let mut reduced = false;
        let mut i = 0usize;
        while i < current.len() && current.len() > 1 {
            let end = (i + chunk).min(current.len());
            let candidate: Vec<usize> = current[..i]
                .iter()
                .chain(current[end..].iter())
                .copied()
                .collect();
            if interesting(&candidate) {
                current = candidate;
                reduced = true;
            } else {
                i = end.max(i + 1);
            }
        }
        if current.len() <= 1 || (!reduced && chunk == 1) {
            break;
        }
        chunk = if reduced { chunk * 2 } else { chunk / 2 };
        chunk = chunk.min(current.len());
    }
    current
}
// this code was written by an ai - end ddmin core

// this code was written by an ai - begin essential tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ddmin_reduces_to_single_culprit() {
        // broken set must still contain the culprit piece
        let r = ddmin(8, &mut |s: &[usize]| s.contains(&3));
        assert_eq!(r, vec![3]);
    }

    #[test]
    fn ddmin_keeps_all_when_every_subset_uninteresting() {
        let r = ddmin(4, &mut |_s| false);
        assert_eq!(r.len(), 4);
    }

    #[test]
    fn ddmin_single_passthrough() {
        assert_eq!(ddmin(1, &mut |_| true), vec![0]);
    }
}
// this code was written by an ai - end essential tests
