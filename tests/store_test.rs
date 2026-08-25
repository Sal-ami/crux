use crux::store::hash::content_hash;

#[test]
fn hash_deterministic() {
    let h1 = content_hash("hello world");
    let h2 = content_hash("hello world");
    assert_eq!(h1, h2);
}

#[test]
fn hash_different_inputs() {
    let h1 = content_hash("hello");
    let h2 = content_hash("world");
    assert_ne!(h1, h2);
}

#[test]
fn hash_empty_string() {
    let h = content_hash("");
    assert!(!h.is_empty());
}
