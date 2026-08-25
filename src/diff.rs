use crate::git;
use std::path::Path;

pub struct DiffEntry {
    pub hash: String,
    pub message: String,
    pub files: Vec<String>,
}

pub fn diff(range: &str, cwd: &Path) -> Vec<DiffEntry> {
    git::log(range, cwd)
        .unwrap_or_default()
        .into_iter()
        .map(|c| DiffEntry {
            hash: c.hash,
            message: c.message,
            files: c.files_changed,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_range_returns_empty() {
        let dir = tempfile::TempDir::new().unwrap();
        let entries = diff("HEAD..HEAD", dir.path());
        assert!(entries.is_empty());
    }
}
