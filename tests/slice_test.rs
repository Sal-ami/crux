use crux::slice::ast;

#[test]
fn extracts_let_bindings() {
    let vars = ast::extract_changed_vars("let x = 1;\nlet y = 2;");
    assert!(vars.contains(&"x".to_string()));
    assert!(vars.contains(&"y".to_string()));
}

#[test]
fn extracts_function_calls() {
    let calls = ast::extract_function_calls("foo(bar())");
    assert!(calls.contains(&"foo".to_string()));
    assert!(calls.contains(&"bar".to_string()));
}

#[test]
fn extracts_imports() {
    let imports = ast::extract_imports("use std::io::Read;");
    assert!(imports.iter().any(|i| i.contains("Read")));
}

#[test]
fn empty_content() {
    let vars = ast::extract_changed_vars("");
    assert!(vars.is_empty());
}
