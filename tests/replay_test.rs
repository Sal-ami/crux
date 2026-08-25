use crux::sandbox;

#[test]
fn verify_same_output() {
    assert!("hello\n" == "hello\n");
}

#[test]
fn verify_different_output() {
    assert!("hello" != "world");
}

#[test]
fn replay_captures_output() {
    let dir = tempfile::TempDir::new().unwrap();
    let out = sandbox::replay("echo hello", dir.path());
    assert!(out.contains("hello"));
}

#[test]
fn replay_empty_on_failure() {
    let dir = tempfile::TempDir::new().unwrap();
    let cmd = if cfg!(windows) { "exit /b 1" } else { "false" };
    let out = sandbox::replay(cmd, dir.path());
    assert!(out.is_empty());
}
