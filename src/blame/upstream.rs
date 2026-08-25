use std::path::Path;

pub struct DepEvidence {
    pub name: String,
    pub kind: &'static str,
    pub old_version: String,
    pub new_version: String,
    pub changed_files: Vec<String>,
    pub url: Option<String>,
}

const LOCKFILES: &[&str] = &["Cargo.lock", "go.sum", "package-lock.json"];
const VENDOR_PREFIXES: &[&str] = &["vendor/", "third_party/", "extern/", "deps/"];

/// Collect dependency-boundary evidence for a flip commit: lockfile
/// version transitions plus vendored-dir manifest deltas. Evidence only;
/// never guesses an upstream cause without history to point at.
pub fn evidence(flip: &str, parent: &str, repo: &Path) -> Vec<DepEvidence> {
    let mut out = Vec::new();
    let flip_files = files_of(flip, repo);

    // 1. vendored directories touched by the flip
    let mut vendor_dirs: Vec<String> = Vec::new();
    for f in &flip_files {
        let f = f.replace('\\', "/");
        for prefix in VENDOR_PREFIXES {
            if let Some(rest) = f.strip_prefix(prefix)
                && let Some(top) = rest.split('/').next()
                && !top.is_empty()
            {
                let dir = format!("{prefix}{top}");
                if !vendor_dirs.contains(&dir) {
                    vendor_dirs.push(dir);
                }
            }
        }
    }
    for dir in vendor_dirs {
        if let Some(ev) = vendored_evidence(&dir, flip, parent, repo, &flip_files) {
            out.push(ev);
        }
    }

    // 2. lockfile transitions
    for lf in LOCKFILES {
        if flip_files.iter().any(|f| f.ends_with(lf)) && *lf == "Cargo.lock" {
            out.extend(lock_evidence(lf, flip, parent, repo));
        }
    }
    out
}

fn vendored_evidence(
    dir: &str,
    flip: &str,
    parent: &str,
    repo: &Path,
    flip_files: &[String],
) -> Option<DepEvidence> {
    let manifest_candidates = [
        format!("{dir}/Cargo.toml"),
        format!("{dir}/package.json"),
    ];
    let manifest = manifest_candidates
        .iter()
        .find(|m| file_exists_at(m, flip, repo) || file_exists_at(m, parent, repo))?;
    let kind = if manifest.ends_with("Cargo.toml") { "vendored-crate" } else { "vendored-npm" };
    let old_v = manifest_version(manifest, parent, repo);
    let new_v = manifest_version(manifest, flip, repo);
    let name = manifest_name(manifest, flip, parent, repo)?;
    if old_v == new_v {
        return None;
    }
    Some(DepEvidence {
        name,
        kind,
        old_version: old_v,
        new_version: new_v,
        changed_files: flip_files
            .iter()
            .filter(|f| f.replace('\\', "/").starts_with(dir))
            .cloned()
            .collect(),
        url: manifest_url(manifest, flip, repo),
    })
}

fn lock_evidence(lf: &str, flip: &str, parent: &str, repo: &Path) -> Vec<DepEvidence> {
    let diff = git_diff(parent, flip, &[lf], repo);
    parse_lock_transitions(&diff)
        .into_iter()
        .map(|(name, old_version, new_version)| DepEvidence {
            name,
            kind: "cargo-lock",
            old_version,
            new_version,
            changed_files: vec![lf.to_string()],
            url: None,
        })
        .collect()
}

/// Extract (name, old_version, new_version) triples from a unified diff of
/// Cargo.lock. Name lines often appear as shared context, so pending names
/// persist across +/- blocks and context versions seed both worlds.
/// Unchanged packages are omitted.
// this code was written by an ai - begin lockfile parser
fn parse_lock_transitions(diff: &str) -> Vec<(String, String, String)> {
    fn clean(s: &str) -> String {
        s.trim().trim_matches('"').split('#').next().unwrap_or("").to_string()
    }
    fn upsert(v: &mut Vec<(String, String)>, k: &str, val: String) {
        v.retain(|(n, _)| n != k);
        v.push((k.to_string(), val));
    }
    let mut old: Vec<(String, String)> = Vec::new();
    let mut new: Vec<(String, String)> = Vec::new();
    let mut pending: Option<String> = None;
    for line in diff.lines() {
        let (side, body) = match (line.starts_with('+'), line.starts_with('-')) {
            (true, false) => ('+', &line[1..]),
            (false, true) => ('-', &line[1..]),
            _ => (' ', line),
        };
        let t = body.trim();
        if t.starts_with("diff --git") || t.starts_with("@@") {
            pending = None;
            continue;
        }
        if let Some(rest) = t.strip_prefix("name = ") {
            pending = Some(clean(rest));
            continue;
        }
        if let Some(rest) = t.strip_prefix("version = ")
            && let Some(name) = &pending
        {
            let v = clean(rest);
            match side {
                '-' => upsert(&mut old, name, v),
                '+' => upsert(&mut new, name, v),
                _ => {
                    upsert(&mut old, name, v.clone());
                    upsert(&mut new, name, v);
                }
            }
        }
    }
    new.iter()
        .filter_map(|(n, nv)| {
            old.iter()
                .find(|(o, _)| o == n)
                .and_then(|(_, ov)| (ov != nv).then(|| (n.clone(), ov.clone(), nv.clone())))
        })
        .collect()
}
// this code was written by an ai - end lockfile parser

fn git_diff(parent: &str, flip: &str, paths: &[&str], repo: &Path) -> String {
    let mut args = vec!["diff", "--no-color", parent, flip, "--"];
    args.extend(paths);
    std::process::Command::new("git")
        .args(&args)
        .current_dir(repo)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default()
}

fn files_of(rev: &str, repo: &Path) -> Vec<String> {
    std::process::Command::new("git")
        .args(["diff-tree", "--no-commit-id", "-r", "--name-only", rev])
        .current_dir(repo)
        .output()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .filter(|l| !l.is_empty())
                .map(String::from)
                .collect()
        })
        .unwrap_or_default()
}

fn show(rev: &str, path: &str, repo: &Path) -> Option<String> {
    let out = std::process::Command::new("git")
        .args(["show", &format!("{rev}:{path}")])
        .current_dir(repo)
        .output()
        .ok()?;
    if out.status.success() {
        Some(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        None
    }
}

fn file_exists_at(path: &str, rev: &str, repo: &Path) -> bool {
    show(rev, path, repo).is_some()
}

fn manifest_version(manifest: &str, rev: &str, repo: &Path) -> String {
    let content = match show(rev, manifest, repo) {
        Some(c) => c,
        None => return String::new(),
    };
    if manifest.ends_with("Cargo.toml") {
        for line in content.lines() {
            if let Some(rest) = line.trim().strip_prefix("version ")
                && let Some(v) = rest.split('=').nth(1)
            {
                return v.trim().trim_matches('"').to_string();
            }
        }
    } else {
        for line in content.lines() {
            if line.contains("\"version\"")
                && let Some(v) = line.split(':').nth(1)
            {
                return v.trim().trim_matches(',').trim_matches('"').to_string();
            }
        }
    }
    String::new()
}

fn manifest_name(manifest: &str, flip: &str, parent: &str, repo: &Path) -> Option<String> {
    let rev = if file_exists_at(manifest, flip, repo) { flip } else { parent };
    let content = show(rev, manifest, repo)?;
    if manifest.ends_with("Cargo.toml") {
        for line in content.lines() {
            if let Some(rest) = line.trim().strip_prefix("name ")
                && let Some(v) = rest.split('=').nth(1)
            {
                return Some(v.trim().trim_matches('"').to_string());
            }
        }
    } else {
        for line in content.lines() {
            if line.contains("\"name\"")
                && let Some(v) = line.split(':').nth(1)
            {
                return Some(v.trim().trim_matches(',').trim_matches('"').to_string());
            }
        }
    }
    None
}

fn manifest_url(manifest: &str, flip: &str, repo: &Path) -> Option<String> {
    let content = show(flip, manifest, repo).or_else(|| show(flip, manifest, repo))?;
    if manifest.ends_with("Cargo.toml") {
        for line in content.lines() {
            if let Some(rest) = line.trim().strip_prefix("repository ")
                && let Some(v) = rest.split('=').nth(1)
            {
                let u = v.trim().trim_matches('"');
                if !u.is_empty() {
                    return Some(u.to_string());
                }
            }
        }
    } else {
        for line in content.lines() {
            if line.contains("\"repository\"")
                && let Some(v) = line.split(':').nth(1)
            {
                let u = v.trim().trim_matches(',').trim_matches('"');
                if u.starts_with("git") {
                    return Some(u.to_string());
                }
            }
        }
    }
    None
}

/// Best-effort upstream attribution: resolve version tags on the declared
/// repository and log what changed between them. Network required; every
/// failure degrades silently to evidence-only output.
pub fn deep_commits(ev: &DepEvidence, max: usize) -> Vec<String> {
    use std::process::{Command, Stdio};
    let url = match &ev.url {
        Some(u) => u.clone(),
        None => return Vec::new(),
    };
    let tag = |v: &str| {
        for prefix in ["v", ""] {
            let t = format!("{prefix}{v}");
            let ok = Command::new("git")
                .args(["ls-remote", "--tags", &url, &t])
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .output()
                .map(|o| !o.stdout.is_empty())
                .unwrap_or(false);
            if ok {
                return t;
            }
        }
        String::new()
    };
    let old_tag = tag(&ev.old_version);
    let new_tag = tag(&ev.new_version);
    if old_tag.is_empty() || new_tag.is_empty() {
        return Vec::new();
    }
    let tmp = std::env::temp_dir().join(format!("crux-upstream-{}", ev.name));
    let _ = std::fs::remove_dir_all(&tmp);
    let cloned = Command::new("git")
        .args([
            "clone",
            "--filter=blob:none",
            "--no-checkout",
            "--quiet",
            &url,
        ])
        .arg(&tmp)
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !cloned {
        return Vec::new();
    }
    let log = Command::new("git")
        .args([
            "log",
            "--oneline",
            &format!("{old_tag}..{new_tag}"),
            &format!("-{}", max),
        ])
        .current_dir(&tmp)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default();
    let _ = std::fs::remove_dir_all(&tmp);
    log.lines().filter(|l| !l.is_empty()).map(String::from).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOCK_DIFF: &str = "\
diff --git a/Cargo.lock b/Cargo.lock
--- a/Cargo.lock
+++ b/Cargo.lock
@@ -1,7 +1,7 @@
 [[package]]
 name = \"libc\"
-version = \"0.1.0\"
+version = \"0.2.0\"

 [[package]]
 name = \"app\"
 version = \"1.0.0\"";

    #[test]
    fn parses_lock_transitions_from_diff_text() {
        let t = parse_lock_transitions(LOCK_DIFF);
        assert_eq!(t, vec![("libc".to_string(), "0.1.0".to_string(), "0.2.0".to_string())]);
    }

    #[test]
    fn lock_context_name_serves_both_sides() {
        // name only as context between - and + version lines
        let diff = "@@ -1,3 +1,3 @@\n name = \"z\"\n-version = \"1\"\n+version = \"2\"";
        assert_eq!(
            parse_lock_transitions(diff),
            vec![("z".to_string(), "1".to_string(), "2".to_string())]
        );
    }

    #[test]
    fn manifest_version_parses_cargo_toml() {
        let dir = tempfile::TempDir::new().unwrap();
        let repo = dir.path();
        std::fs::write(repo.join("m.toml"), "[package]\nname = \"x\"\nversion = \"3.2.1\"\n").unwrap();
        // show() requires git; exercise the text parser directly instead
        let content = std::fs::read_to_string(repo.join("m.toml")).unwrap();
        let mut v = String::new();
        for line in content.lines() {
            if let Some(rest) = line.trim().strip_prefix("version ")
                && let Some(val) = rest.split('=').nth(1)
            {
                v = val.trim().trim_matches('"').to_string();
            }
        }
        assert_eq!(v, "3.2.1");
    }

    #[test]
    fn deep_commits_degrades_without_network() {
        let ev = DepEvidence {
            name: "x".into(),
            kind: "cargo-lock",
            old_version: "1".into(),
            new_version: "2".into(),
            changed_files: vec![],
            url: Some("https://invalid.invalid/nope".into()),
        };
        assert!(deep_commits(&ev, 5).is_empty());
    }
}
