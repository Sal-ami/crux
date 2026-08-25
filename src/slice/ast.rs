pub fn extract_changed_vars(content: &str) -> Vec<String> {
    let mut vars = Vec::new();
    for line in content.lines() {
        for prefix in &["let ", "const ", "var ", "mut "] {
            let mut search_from = 0;
            while let Some(idx) = line[search_from..].find(prefix) {
                let start = search_from + idx + prefix.len();
                if let Some(name) = line[start..].split(|c: char| !c.is_alphanumeric() && c != '_').next() {
                    let name = name.trim();
                    if !name.is_empty() && !super::KEYWORDS.contains(&name) {
                        vars.push(name.to_string());
                    }
                }
                search_from = start;
            }
        }
        let trimmed = line.trim();
        if !super::KEYWORDS.iter().any(|k| trimmed.starts_with(k)) {
            for word in line.split(|c: char| !c.is_alphanumeric() && c != '_') {
                let word = word.trim();
                if !word.is_empty() && !super::KEYWORDS.contains(&word) && trimmed.starts_with(word) && line.contains('=') {
                    vars.push(word.to_string());
                }
            }
        }
    }
    vars.sort();
    vars.dedup();
    vars
}

pub fn extract_function_calls(content: &str) -> Vec<String> {
    let mut calls = Vec::new();
    let mut current = String::new();
    for c in content.chars() {
        if c.is_alphanumeric() || c == '_' {
            current.push(c);
        } else {
            if !current.is_empty() && c == '(' && !super::KEYWORDS.contains(&current.as_str()) {
                calls.push(current.clone());
            }
            current.clear();
        }
    }
    calls.sort();
    calls.dedup();
    calls
}

pub fn extract_imports(content: &str) -> Vec<String> {
    let mut imports = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("use ") {
            let path = rest.trim_end_matches(';').trim();
            if !path.is_empty() {
                imports.push(path.to_string());
            }
        }
        for prefix in &["from \"", "from '", "require(\"", "require('"] {
            let quote = if prefix.ends_with('"') { '"' } else { '\'' };
            if let Some(idx) = trimmed.find(prefix) {
                let start = idx + prefix.len();
                if let Some(end) = trimmed[start..].find(quote) {
                    imports.push(trimmed[start..start + end].to_string());
                }
            }
        }
    }
    imports.sort();
    imports.dedup();
    imports
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_let_bindings() {
        let vars = extract_changed_vars("let x = 1; let y = 2;");
        assert!(vars.contains(&"x".to_string()));
        assert!(vars.contains(&"y".to_string()));
    }

    #[test]
    fn extracts_reassignment() {
        let vars = extract_changed_vars("x = 5;");
        assert!(vars.contains(&"x".to_string()));
    }

    #[test]
    fn extracts_function_calls() {
        let calls = extract_function_calls("foo(bar(), baz(1))");
        assert!(calls.contains(&"foo".to_string()));
        assert!(calls.contains(&"bar".to_string()));
        assert!(calls.contains(&"baz".to_string()));
    }

    #[test]
    fn extracts_rust_use() {
        let imports = extract_imports("use std::collections::HashMap;");
        assert!(imports.iter().any(|i| i.contains("HashMap")));
    }

    #[test]
    fn empty_content() {
        let vars = extract_changed_vars("");
        assert!(vars.is_empty());
    }
}
