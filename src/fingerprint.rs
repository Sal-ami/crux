use sha2::{Digest, Sha256};

#[derive(Debug)]
pub struct Fingerprint {
    pub hash: String,
    pub tokens: Vec<String>,
}

pub fn fingerprint(cmd: &str) -> Fingerprint {
    let tokens: Vec<String> = cmd.split_whitespace().map(String::from).collect();
    let mut hasher = Sha256::new();
    for token in &tokens {
        hasher.update(token.as_bytes());
    }
    let hash = format!("{:x}", hasher.finalize());
    Fingerprint { hash, tokens }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_input_same_hash() {
        let a = fingerprint("cargo test");
        let b = fingerprint("cargo test");
        assert_eq!(a.hash, b.hash);
    }

    #[test]
    fn different_input_different_hash() {
        let a = fingerprint("cargo test");
        let b = fingerprint("cargo build");
        assert_ne!(a.hash, b.hash);
    }

    #[test]
    fn tokens_split_on_whitespace() {
        let fp = fingerprint("  cargo   test  ");
        assert_eq!(fp.tokens, vec!["cargo", "test"]);
    }

    #[test]
    fn empty_input() {
        let fp = fingerprint("");
        assert!(fp.tokens.is_empty());
        assert!(!fp.hash.is_empty());
    }
}
