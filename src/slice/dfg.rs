use std::collections::{HashMap, HashSet};

pub struct DfgNode {
    pub var: String,
    pub line: usize,
    pub deps: Vec<String>,
}

pub struct Dfg {
    pub nodes: Vec<DfgNode>,
    pub def_map: HashMap<String, Vec<usize>>,
    pub use_map: HashMap<String, Vec<usize>>,
}

pub fn build_dfg(content: &str) -> Dfg {
    let lines: Vec<&str> = content.lines().collect();
    let mut nodes = Vec::new();
    let mut def_map: HashMap<String, Vec<usize>> = HashMap::new();
    let mut use_map: HashMap<String, Vec<usize>> = HashMap::new();

    for (i, line) in lines.iter().enumerate() {
        let mut defined = Vec::new();
        let mut used = Vec::new();
        let mut remaining = *line;
        loop {
            let t = remaining.trim_start();
            if t.is_empty() {
                break;
            }
            let mut found = false;
            for prefix in &["let ", "const ", "var ", "mut "] {
                if let Some(rest) = t.strip_prefix(prefix) {
                    if let Some(name) = rest.split(|c: char| !c.is_alphanumeric() && c != '_').next() {
                        let name = name.trim().to_string();
                        if !name.is_empty() && !super::KEYWORDS.contains(&name.as_str()) {
                            def_map.entry(name.clone()).or_default().push(i);
                            defined.push(name);
                        }
                    }
                    let consumed = line.len() - rest.len();
                    remaining = &line[consumed..];
                    found = true;
                    break;
                }
            }
            if !found {
                break;
            }
        }

        for word in line.split(|c: char| !c.is_alphanumeric() && c != '_') {
            let word = word.trim();
            if word.is_empty() || super::KEYWORDS.contains(&word) || defined.contains(&word.to_string()) {
                continue;
            }
            used.push(word.to_string());
            use_map.entry(word.to_string()).or_default().push(i);
        }

        for def in &defined {
            nodes.push(DfgNode {
                var: def.clone(),
                line: i,
                deps: used.clone(),
            });
        }
    }

    Dfg { nodes, def_map, use_map }
}

pub fn trace_back(dfg: &Dfg, target_var: &str) -> Vec<usize> {
    let mut visited = HashSet::new();
    let mut stack: Vec<String> = vec![target_var.to_string()];
    let mut lines = Vec::new();
    while let Some(var) = stack.pop() {
        if visited.contains(&var) {
            continue;
        }
        visited.insert(var.clone());
        if let Some(defs) = dfg.def_map.get(&var) {
            for &line_idx in defs {
                lines.push(line_idx);
                for node in &dfg.nodes {
                    if node.var == var && node.line == line_idx {
                        for dep in &node.deps {
                            stack.push(dep.clone());
                        }
                        break;
                    }
                }
            }
        }
    }
    lines.sort();
    lines.dedup();
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_dfg() {
        let content = "let x = 1;\nlet y = x + 1;\nlet z = y;";
        let dfg = build_dfg(content);
        assert!(dfg.nodes.len() >= 2);
        assert!(dfg.def_map.contains_key("x"));
        assert!(dfg.def_map.contains_key("y"));
    }

    #[test]
    fn traces_deps() {
        let content = "let a = 1;\nlet b = a;\nlet c = b;";
        let dfg = build_dfg(content);
        let lines = trace_back(&dfg, "c");
        assert!(!lines.is_empty());
    }
}
