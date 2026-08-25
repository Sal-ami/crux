use std::path::Path;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Guardian {
    pub name: String,
    pub behavior: String,
    pub expect: String,
}

fn path(repo: &Path) -> std::path::PathBuf {
    repo.join(".crux").join("guardians.json")
}

pub fn list(repo: &Path) -> Vec<Guardian> {
    std::fs::read_to_string(path(repo))
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default()
}

pub fn add(repo: &Path, g: Guardian) -> crate::error::Result<()> {
    let mut all = list(repo);
    all.retain(|x| x.name != g.name);
    all.push(g);
    save(repo, &all)
}

pub fn remove(repo: &Path, name: &str) -> crate::error::Result<bool> {
    let mut all = list(repo);
    let before = all.len();
    all.retain(|x| x.name != name);
    if all.len() == before {
        return Ok(false);
    }
    save(repo, &all)?;
    Ok(true)
}

fn save(repo: &Path, all: &[Guardian]) -> crate::error::Result<()> {
    std::fs::create_dir_all(repo.join(".crux"))?;
    std::fs::write(path(repo), serde_json::to_string(all)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn add_list_remove_roundtrip() {
        let dir = TempDir::new().unwrap();
        add(
            dir.path(),
            Guardian {
                name: "g1".into(),
                behavior: "cargo test".into(),
                expect: "pass".into(),
            },
        )
        .unwrap();
        assert_eq!(list(dir.path()).len(), 1);
        add(
            dir.path(),
            Guardian {
                name: "g1".into(),
                behavior: "other".into(),
                expect: "pass".into(),
            },
        )
        .unwrap();
        assert_eq!(list(dir.path()).len(), 1);
        assert!(remove(dir.path(), "g1").unwrap());
        assert!(!remove(dir.path(), "g1").unwrap());
        assert!(list(dir.path()).is_empty());
    }
}
