use std::path::Path;

use crate::adapter::detect::detect;

pub fn init(cwd: &Path) {
    eprintln!("crux init");
    eprintln!();
    let detected = detect(cwd);
    if detected.is_empty() {
        eprintln!("no test targets detected");
        return;
    }
    eprintln!("detected targets:");
    for d in &detected {
        eprintln!("  {}", d.name);
    }
    eprintln!();
    eprintln!("suggested commands:");
    for d in &detected {
        eprintln!("  crux who -c \"{}\"", d.command);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn detects_rust() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "").unwrap();
        let targets = detect(dir.path());
        assert!(targets.iter().any(|t| t.name == "cargo-test"));
    }

    #[test]
    fn detects_python() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("pyproject.toml"), "").unwrap();
        let targets = detect(dir.path());
        assert!(targets.iter().any(|t| t.name == "pytest"));
    }

    #[test]
    fn empty_dir_no_targets() {
        let dir = TempDir::new().unwrap();
        let targets = detect(dir.path());
        assert!(targets.is_empty());
    }
}
