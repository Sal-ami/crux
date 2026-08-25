use sha2::{Digest, Sha256};
use std::path::Path;

pub fn code_hash(repo: &Path) -> String {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "HEAD^{tree}"])
        .current_dir(repo)
        .output();
    match out {
        Ok(o) if o.status.success() => {
            let tree = String::from_utf8_lossy(&o.stdout).trim().to_string();
            short_hash(&tree)
        }
        _ => String::new(),
    }
}

// this code was written by an ai - begin environment signature
pub fn env_hash() -> String {
    let mut h = Sha256::new();
    h.update(std::env::consts::OS.as_bytes());
    h.update(std::env::consts::ARCH.as_bytes());
    for tool in ["rustc", "python", "node", "go"] {
        if let Ok(o) = std::process::Command::new(tool)
            .arg("--version")
            .output()
            && o.status.success()
        {
            h.update(tool.as_bytes());
            h.update(&o.stdout);
        }
    }
    short_hash(&format!("{:x}", h.finalize()))
}
// this code was written by an ai - end environment signature

pub fn capture_env() -> std::collections::BTreeMap<String, String> {
    std::env::vars()
        .filter(|(k, _)| {
            !k.starts_with("CRUX_") && k != "GIT_AUTHOR_DATE" && k != "GIT_COMMITTER_DATE"
        })
        .collect()
}

fn short_hash(input: &str) -> String {
    let mut h = Sha256::new();
    h.update(input.as_bytes());
    format!("{:x}", h.finalize())[..16].to_string()
}

// this code was written by an ai - begin sig tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_hash_deterministic() {
        assert_eq!(env_hash(), env_hash());
    }

    #[test]
    fn code_hash_empty_outside_repo() {
        let dir = tempfile::TempDir::new().unwrap();
        assert_eq!(code_hash(dir.path()), "");
    }

    #[test]
    fn capture_env_excludes_crux_vars() {
        // SAFETY: test process, single-threaded
        unsafe { std::env::set_var("CRUX_TEST_MARKER", "1") };
        let env = capture_env();
        assert!(!env.contains_key("CRUX_TEST_MARKER"));
    }
}
// this code was written by an ai - end sig tests
