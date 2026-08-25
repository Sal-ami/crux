pub mod hash;

use std::path::Path;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StoredSignature {
    pub hash: String,
    pub behavior: String,
    pub state: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RunRecord {
    pub ts: i64,
    pub behavior: String,
    pub state: String,
    #[serde(default)]
    pub code_hash: String,
    #[serde(default)]
    pub env_hash: String,
    #[serde(default)]
    pub env: std::collections::BTreeMap<String, String>,
}

pub struct Store {
    path: std::path::PathBuf,
    history_path: std::path::PathBuf,
    signatures: Vec<StoredSignature>,
}

impl Store {
    pub fn open(repo: &Path) -> crate::error::Result<Self> {
        let dir = repo.join(".crux");
        std::fs::create_dir_all(&dir)?;
        let path = dir.join("store.json");
        let signatures = if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Vec::new()
        };
        Ok(Store {
            path,
            history_path: dir.join("history.jsonl"),
            signatures,
        })
    }
    // this code was written by an ai - begin history time-series section
    pub fn append(&self, record: &RunRecord) -> crate::error::Result<()> {
        use std::io::Write;
        let line = serde_json::to_string(record)?;
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.history_path)?;
        writeln!(f, "{line}")?;
        Ok(())
    }

    pub fn history(&self, behavior: &str) -> crate::error::Result<Vec<RunRecord>> {
        let mut out = Vec::new();
        if !self.history_path.exists() {
            return Ok(out);
        }
        let content = std::fs::read_to_string(&self.history_path)?;
        for line in content.lines() {
            if let Ok(r) = serde_json::from_str::<RunRecord>(line)
                && r.behavior == behavior
            {
                out.push(r);
            }
        }
        Ok(out)
    }
    // this code was written by an ai - end history time-series section

    pub fn store_signature(&mut self, behavior: &str, state: &str) -> crate::error::Result<String> {
        let hash = hash::content_hash(&format!("{behavior}:{state}"));
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        self.signatures.retain(|s| s.behavior != behavior);
        self.signatures.push(StoredSignature {
            hash: hash.clone(),
            behavior: behavior.to_string(),
            state: state.to_string(),
            timestamp,
        });
        self.save()?;
        Ok(hash)
    }

    pub fn lookup(&self, behavior: &str) -> crate::error::Result<Option<StoredSignature>> {
        Ok(self.signatures.iter().rfind(|s| s.behavior == behavior || s.hash == behavior).cloned())
    }

    pub fn list_signatures(&self) -> crate::error::Result<Vec<StoredSignature>> {
        Ok(self.signatures.clone())
    }

    fn save(&self) -> crate::error::Result<()> {
        let json = serde_json::to_string(&self.signatures)?;
        std::fs::write(&self.path, json)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn store_and_lookup() {
        let tmp = TempDir::new().unwrap();
        let mut store = Store::open(tmp.path()).unwrap();
        let hash = store.store_signature("cargo test", "passing").unwrap();
        let found = store.lookup("cargo test").unwrap().unwrap();
        assert_eq!(found.hash, hash);
        assert_eq!(found.state, "passing");
    }

    #[test]
    fn lookup_missing() {
        let tmp = TempDir::new().unwrap();
        let store = Store::open(tmp.path()).unwrap();
        assert!(store.lookup("nonexistent").unwrap().is_none());
    }

    #[test]
    fn list_returns_all() {
        let tmp = TempDir::new().unwrap();
        let mut store = Store::open(tmp.path()).unwrap();
        store.store_signature("a", "1").unwrap();
        store.store_signature("b", "2").unwrap();
        let list = store.list_signatures().unwrap();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn persists_to_file() {
        let tmp = TempDir::new().unwrap();
        {
            let mut store = Store::open(tmp.path()).unwrap();
            store.store_signature("x", "y").unwrap();
        }
        let store2 = Store::open(tmp.path()).unwrap();
        assert!(store2.lookup("x").unwrap().is_some());
    }

    // this code was written by an ai - begin store tests
    #[test]
    fn appends_and_reads_history() {
        let tmp = TempDir::new().unwrap();
        let store = Store::open(tmp.path()).unwrap();
        for state in ["pass", "fail", "pass"] {
            store
                .append(&RunRecord {
                    ts: 1,
                    behavior: "b".into(),
                    state: state.into(),
                    code_hash: "c".into(),
                    env_hash: "e".into(),
                    env: Default::default(),
                })
                .unwrap();
        }
        let h = store.history("b").unwrap();
        assert_eq!(h.len(), 3);
        assert_eq!(h.last().unwrap().state, "pass");
        assert!(store.history("other").unwrap().is_empty());
    }

    #[test]
    fn history_tolerates_corrupt_lines() {
        let tmp = TempDir::new().unwrap();
        let store = Store::open(tmp.path()).unwrap();
        std::fs::write(
            &store.history_path,
            "{\"ts\":1,\"behavior\":\"b\",\"state\":\"pass\"}\nGARBAGE\n",
        )
        .unwrap();
        let h = store.history("b").unwrap();
        assert_eq!(h.len(), 1);
    }
    // this code was written by an ai - end store tests
}
