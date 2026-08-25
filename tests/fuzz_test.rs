use proptest::prelude::*;

proptest! {
    #[test]
    fn ast_extract_vars_no_panic(input in ".*") {
        let _ = crux::slice::ast::extract_changed_vars(&input);
    }

    #[test]
    fn ast_extract_calls_no_panic(input in ".*") {
        let _ = crux::slice::ast::extract_function_calls(&input);
    }

    #[test]
    fn ast_extract_imports_no_panic(input in ".*") {
        let _ = crux::slice::ast::extract_imports(&input);
    }

    #[test]
    fn dfg_build_no_panic(input in ".*") {
        let _ = crux::slice::dfg::build_dfg(&input);
    }

    #[test]
    fn ts_extract_vars_no_panic(input in ".*", path in "[a-z]+\\.(rs|py|json)") {
        let _ = crux::slice::tree_sitter::extract_changed_vars(&input, &path);
    }

    #[test]
    fn ts_extract_calls_no_panic(input in ".*", path in "[a-z]+\\.rs") {
        let _ = crux::slice::tree_sitter::extract_function_calls(&input, &path);
    }

    #[test]
    fn fingerprint_no_panic(input in ".*") {
        let _ = crux::fingerprint::fingerprint(&input);
    }

    #[test]
    fn ddmin_no_panic(input in "[a-z ]{0,100}") {
        let candidates: Vec<String> = input.split_whitespace().map(String::from).collect();
        if !candidates.is_empty() {
            let dir = tempfile::TempDir::new().unwrap();
            let _ = crux::min::ddmin::ddmin(&candidates, "HEAD~1..HEAD", "echo ok", dir.path());
        }
    }
}
