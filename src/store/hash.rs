use sha2::{Digest, Sha256};

pub fn content_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic() {
        let h1 = content_hash("hello");
        let h2 = content_hash("hello");
        assert_eq!(h1, h2);
    }

    #[test]
    fn different_for_different_content() {
        let h1 = content_hash("hello");
        let h2 = content_hash("world");
        assert_ne!(h1, h2);
    }
}
