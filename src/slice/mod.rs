pub mod ast;
pub mod dfg;
pub mod filter;
pub mod tree_sitter;

use std::path::Path;

pub(crate) static KEYWORDS: &[&str] = &[
    "let", "const", "var", "mut", "fn", "if", "else", "while", "for", "return",
    "struct", "enum", "impl", "use", "mod", "pub", "self", "true", "false",
];

pub struct SliceResult {
    pub file: String,
    pub caused_by: Vec<String>,
}

pub fn slice(suspects: &[String], cwd: &Path) -> Vec<SliceResult> {
    let mut results = Vec::new();
    for file in suspects {
        let path = cwd.join(file);
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let changed_vars = tree_sitter::extract_changed_vars(&content, file);
        let dfg_graph = dfg::build_dfg(&content);
        let mut dep_lines: Vec<usize> = Vec::new();
        for var in &changed_vars {
            dep_lines.extend(dfg::trace_back(&dfg_graph, var));
        }
        dep_lines.sort();
        dep_lines.dedup();
        let deps: Vec<String> = dep_lines.into_iter().map(|l| l.to_string()).collect();
        let filtered = filter::filter_noise(&deps, &content);
        let caused_by: Vec<String> = filtered
            .into_iter()
            .filter(|l| *l != file.as_str())
            .collect();
        results.push(SliceResult {
            file: file.clone(),
            caused_by,
        });
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_suspects() {
        let dir = tempfile::TempDir::new().unwrap();
        let r = slice(&[], dir.path());
        assert!(r.is_empty());
    }

    #[test]
    fn nonexistent_file() {
        let dir = tempfile::TempDir::new().unwrap();
        let r = slice(&["no_such_file.rs".into()], dir.path());
        assert!(r.is_empty());
    }
}
