pub mod json;
pub mod terminal;

pub struct Report<'a> {
    pub hash: &'a str,
    pub message: &'a str,
    pub suspects: &'a [String],
    pub roots: Vec<&'a String>,
    pub deps: Vec<(&'a String, &'a Vec<String>)>,
    pub iterations: usize,
}

pub fn render(report: &Report, output: Option<&str>) -> String {
    match output {
        Some("json") => json::render(report),
        _ => terminal::render(report),
    }
}

pub fn render_diff(entries: &[crate::diff::DiffEntry], output: Option<&str>) -> String {
    match output {
        Some("json") => {
            let mut out = String::from("[");
            for (i, entry) in entries.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&format!(
                    "{{\"commit\":\"{}\",\"message\":\"{}\",\"files\":[",
                    entry.hash,
                    entry.message.replace('\\', "\\\\").replace('"', "\\\"")
                ));
                for (j, file) in entry.files.iter().enumerate() {
                    if j > 0 {
                        out.push(',');
                    }
                    out.push_str(&format!("\"{}\"", file.replace('\\', "\\\\").replace('"', "\\\"")));
                }
                out.push_str("]}");
            }
            out.push_str("]\n");
            out
        }
        _ => {
            let mut out = String::new();
            for entry in entries {
                out.push_str(&format!(
                    "{} {}\n",
                    &entry.hash[..12.min(entry.hash.len())],
                    entry.message
                ));
                for file in &entry.files {
                    out.push_str(&format!("  M {file}\n"));
                }
            }
            out
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_report() -> Report<'static> {
        let suspects: &'static [String] =
            Box::leak(Box::new(vec!["src/main.rs".to_string(), "src/lib.rs".to_string()]));
        let root: &'static String = Box::leak(Box::new("src/main.rs".to_string()));
        let dep_file: &'static String = Box::leak(Box::new("src/lib.rs".to_string()));
        let dep_causes: &'static Vec<String> =
            Box::leak(Box::new(vec!["src/main.rs".to_string()]));
        Report {
            hash: "abc123def456789",
            message: "fix bug \"in\" module",
            suspects,
            roots: vec![root],
            deps: vec![(dep_file, dep_causes)],
            iterations: 3,
        }
    }

    #[test]
    fn terminal_has_structure_and_ansi() {
        let out = render(&test_report(), None);
        assert!(out.contains("\x1b[1mcommit\x1b[0m"), "missing bold commit");
        assert!(out.contains("\x1b[1msuspects\x1b[0m"), "missing bold suspects");
        assert!(out.contains("\x1b[1mroot causes\x1b[0m"), "missing bold roots");
        assert!(out.contains("\x1b[1mdependency chain\x1b[0m"), "missing bold deps");
        assert!(out.contains("abc123def456"), "missing hash");
        assert!(out.contains("src/main.rs"), "missing suspect");
        assert!(out.contains("3"), "missing iterations");
    }

    #[test]
    fn json_valid_and_escapes() {
        let out = render(&test_report(), Some("json"));
        assert!(out.contains("\"commit\""));
        assert!(out.contains("\"suspects\""));
        assert!(out.contains("\"root_causes\""));
        assert!(out.contains("\"dependency_chain\""));
        assert!(out.contains("\\\"in\\\""), "quotes not escaped");
    }

    #[test]
    fn default_output_is_terminal() {
        let t = render(&test_report(), None);
        let j = render(&test_report(), Some("json"));
        assert_ne!(t, j);
        assert!(t.contains("\x1b["));
    }
}
