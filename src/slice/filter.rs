pub fn filter_noise(candidates: &[String], content: &str) -> Vec<String> {
    let lines: Vec<&str> = content.lines().collect();
    candidates
        .iter()
        .filter(|c| {
            if let Ok(idx) = c.parse::<usize>()
                && idx < lines.len()
            {
                let line = lines[idx].trim();
                if line.is_empty() || line.starts_with("//") || line.starts_with('#')
                    || line.starts_with("/*") || line.starts_with("*")
                {
                    return false;
                }
                if is_reorder(candidates, idx, &lines) {
                    return false;
                }
            }
            true
        })
        .cloned()
        .collect()
}

fn is_reorder(all: &[String], idx: usize, lines: &[&str]) -> bool {
    if idx >= lines.len() {
        return false;
    }
    let line = lines[idx].trim();
    let is_let = line.starts_with("let ") || line.starts_with("const ") || line.starts_with("var ");
    if !is_let {
        return false;
    }
    let siblings: Vec<usize> = all
        .iter()
        .filter_map(|s| s.parse::<usize>().ok())
        .filter(|&i| i < lines.len() && i != idx)
        .filter(|&i| {
            let l = lines[i].trim();
            l.starts_with("let ") || l.starts_with("const ") || l.starts_with("var ")
        })
        .collect();
    if siblings.is_empty() {
        return false;
    }
    lines[idx].trim() == lines[siblings[0]].trim()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_comments() {
        let content = "// comment\nlet x = 1;\n# python comment\nlet y = 2;";
        let candidates = vec!["0".into(), "1".into(), "2".into(), "3".into()];
        let filtered = filter_noise(&candidates, content);
        assert!(!filtered.contains(&"0".to_string()));
        assert!(!filtered.contains(&"2".to_string()));
    }

    #[test]
    fn keeps_real_lines() {
        let content = "let x = 1;\nlet y = x + 1;";
        let candidates = vec!["0".into(), "1".into()];
        let filtered = filter_noise(&candidates, content);
        assert_eq!(filtered.len(), 2);
    }
}
