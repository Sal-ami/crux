use std::path::Path;

pub struct Detected {
    pub name: &'static str,
    pub command: String,
}

pub fn detect(cwd: &Path) -> Vec<Detected> {
    let mut results = Vec::new();

    if cwd.join("Cargo.toml").exists() {
        results.push(Detected {
            name: "cargo-test",
            command: "cargo test".into(),
        });
    }
    if cwd.join("pyproject.toml").exists()
        || cwd.join("setup.py").exists()
        || cwd.join("pytest.ini").exists()
    {
        let cmd = if cwd.join("tests").exists() {
            "pytest tests/".into()
        } else {
            "pytest".into()
        };
        results.push(Detected { name: "pytest", command: cmd });
    }
    if cwd.join("go.mod").exists() {
        results.push(Detected {
            name: "go-test",
            command: "go test ./...".into(),
        });
    }
    if cwd.join("package.json").exists() {
        results.push(Detected {
            name: "npm-test",
            command: "npm test".into(),
        });
    }
    if cwd.join("pom.xml").exists() || cwd.join("build.gradle").exists() {
        results.push(Detected {
            name: "maven-test",
            command: "mvn test".into(),
        });
    }
    if cwd.join("Makefile").exists() {
        let content = std::fs::read_to_string(cwd.join("Makefile")).unwrap_or_default();
        if content.contains("test:") {
            results.push(Detected {
                name: "make-test",
                command: "make test".into(),
            });
        }
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn detects_cargo() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "").unwrap();
        let d = detect(dir.path());
        assert!(d.iter().any(|x| x.name == "cargo-test"));
    }

    #[test]
    fn detects_go() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("go.mod"), "").unwrap();
        let d = detect(dir.path());
        assert!(d.iter().any(|x| x.name == "go-test"));
    }

    #[test]
    fn detects_pytest() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("pyproject.toml"), "").unwrap();
        let d = detect(dir.path());
        assert!(d.iter().any(|x| x.name == "pytest"));
    }

    #[test]
    fn empty_dir_none() {
        let dir = TempDir::new().unwrap();
        let d = detect(dir.path());
        assert!(d.is_empty());
    }
}
