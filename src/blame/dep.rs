use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug)]
pub struct DepInfo {
    pub package: String,
    pub version: String,
    pub file: String,
    pub vendor_changed_at: Option<String>,
}

fn find_vendor_root(file: &str, repo: &Path) -> Option<PathBuf> {
    let path = repo.join(file);
    let mut cur = path.parent()?.to_path_buf();
    loop {
        if cur.file_name().is_some_and(|n| n == "vendor") {
            return Some(cur);
        }
        if !cur.pop() {
            return None;
        }
    }
}

fn package_from_path(vendor_root: &Path, file: &str, repo: &Path) -> Option<String> {
    let path = repo.join(file);
    let relative = path.strip_prefix(vendor_root).ok()?;
    Some(
        relative
            .components()
            .next()?
            .as_os_str()
            .to_str()?
            .to_string(),
    )
}

fn parse_cargo_lock(repo: &Path) -> Vec<(String, String)> {
    let lock = repo.join("Cargo.lock");
    let content = match std::fs::read_to_string(&lock) {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let mut packages = Vec::new();
    let mut name = String::new();
    let mut version = String::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("name = ") {
            name = trimmed
                .trim_start_matches("name = ")
                .trim_matches('"')
                .to_string();
        } else if trimmed.starts_with("version = ") {
            version = trimmed
                .trim_start_matches("version = ")
                .trim_matches('"')
                .to_string();
        } else if trimmed.is_empty() || trimmed == "[[package]]" {
            if !name.is_empty() && !version.is_empty() {
                packages.push((std::mem::take(&mut name), std::mem::take(&mut version)));
            }
            name.clear();
            version.clear();
        }
    }
    if !name.is_empty() && !version.is_empty() {
        packages.push((name, version));
    }
    packages
}

fn find_vendor_change_commit(package: &str, repo: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["log", "--oneline", "--follow", "-1", "--"])
        .arg(format!("vendor/{package}"))
        .current_dir(repo)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let line = String::from_utf8_lossy(&output.stdout);
    let first_line = line.lines().next()?;
    let hash = first_line.split_whitespace().next()?;
    Some(hash.to_string())
}

pub fn trace_upstream(file: &str, repo: &Path) -> Option<DepInfo> {
    let vendor_root = find_vendor_root(file, repo)?;
    let package = package_from_path(&vendor_root, file, repo)?;
    let packages = parse_cargo_lock(repo);
    let version = packages
        .iter()
        .find(|(n, _)| n == &package)
        .map(|(_, v)| v.to_string())
        .unwrap_or_default();
    let vendor_changed_at = find_vendor_change_commit(&package, repo);
    Some(DepInfo {
        package,
        version,
        file: file.to_string(),
        vendor_changed_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn finds_vendored_package() {
        let tmp = TempDir::new().unwrap();
        let repo = tmp.path();
        Command::new("git").args(["init"]).current_dir(repo).output().unwrap();
        std::fs::create_dir_all(repo.join("vendor/serde/src")).unwrap();
        std::fs::write(repo.join("vendor/serde/src/lib.rs"), "").unwrap();
        std::fs::write(
            repo.join("Cargo.lock"),
            "[[package]]\nname = \"serde\"\nversion = \"1.0.0\"\n",
        )
        .unwrap();
        Command::new("git").args(["-c", "user.name=test", "-c", "user.email=t@t.t", "add", "."]).current_dir(repo).output().unwrap();
        Command::new("git").args(["-c", "user.name=test", "-c", "user.email=t@t.t", "commit", "-m", "add vendor"]).current_dir(repo).output().unwrap();
        let dep = trace_upstream("vendor/serde/src/lib.rs", repo).unwrap();
        assert_eq!(dep.package, "serde");
        assert_eq!(dep.version, "1.0.0");
        assert!(dep.vendor_changed_at.is_some());
    }

    #[test]
    fn returns_none_for_non_vendor() {
        let tmp = TempDir::new().unwrap();
        let dep = trace_upstream("src/main.rs", tmp.path());
        assert!(dep.is_none());
    }

    #[test]
    fn handles_missing_lockfile() {
        let tmp = TempDir::new().unwrap();
        let repo = tmp.path();
        std::fs::create_dir_all(repo.join("vendor/foo/src")).unwrap();
        std::fs::write(repo.join("vendor/foo/src/lib.rs"), "").unwrap();
        let dep = trace_upstream("vendor/foo/src/lib.rs", repo).unwrap();
        assert_eq!(dep.package, "foo");
        assert_eq!(dep.version, "");
    }
}
