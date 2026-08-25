use super::Report;

pub fn render(report: &Report) -> String {
    let mut out = String::new();
    let h = &report.hash[..12.min(report.hash.len())];

    out.push_str("\x1b[1mcommit\x1b[0m ");
    out.push_str(h);
    out.push(' ');
    out.push_str(report.message);
    out.push('\n');
    out.push('\n');

    out.push_str(&format!(
        "\x1b[1msuspects\x1b[0m ({} iterations)\n",
        report.iterations
    ));
    for s in report.suspects {
        out.push_str(&format!("  {}\n", s));
    }
    out.push('\n');

    if !report.deps.is_empty() {
        out.push_str("\x1b[1mdependency chain\x1b[0m\n");
        for (file, caused_by) in &report.deps {
            out.push_str(&format!(
                "  \x1b[33m{}\x1b[0m caused by {}\n",
                file,
                caused_by.join(", ")
            ));
        }
        out.push('\n');
    }

    if !report.roots.is_empty() {
        out.push_str("\x1b[1mroot causes\x1b[0m\n");
        for r in &report.roots {
            out.push_str(&format!("  \x1b[31m{}\x1b[0m\n", r));
        }
    }

    out
}
