use crux::min::ddmin::ddmin_set;

#[test]
fn ddmin_reduces_to_minimal() {
    let input: Vec<String> = (0..16).map(|i| i.to_string()).collect();
    let (result, iters) = ddmin_set(&input, &|s| s.len() < 16);
    assert!(result.len() < 16, "expected reduction from 16, got {}", result.len());
    assert!(iters > 0);
}

#[test]
fn ddmin_empty_input() {
    let (result, iters) = ddmin_set(&[], &|_| false);
    assert!(result.is_empty());
    assert_eq!(iters, 0);
}

#[test]
fn ddmin_already_minimal() {
    let input = vec!["a".to_string()];
    let (result, _) = ddmin_set(&input, &|s| s.is_empty());
    assert_eq!(result, vec!["a"]);
}

#[test]
fn ddmin_preserves_all_when_needed() {
    let input: Vec<String> = (0..4).map(|i| i.to_string()).collect();
    let (result, _) = ddmin_set(&input, &|s| s.is_empty());
    assert_eq!(result.len(), 4);
}

#[test]
fn ddmin_with_string_fn() {
    let input: Vec<String> = vec!["x".into(), "y".into(), "z".into()];
    let (result, _) = ddmin_set(&input, &|s| s.len() >= 2);
    assert!(result.len() <= 3);
    assert!(!result.is_empty());
}
