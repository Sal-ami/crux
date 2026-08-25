use super::Report;

fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

pub fn render(report: &Report) -> String {
    let h = &report.hash[..12.min(report.hash.len())];

    let suspects: String = report
        .suspects
        .iter()
        .map(|s| format!("    {}", esc(s)))
        .collect::<Vec<_>>()
        .join(",\n");

    let roots: String = report
        .roots
        .iter()
        .map(|r| format!("    {}", esc(r)))
        .collect::<Vec<_>>()
        .join(",\n");

    let deps: String = report
        .deps
        .iter()
        .map(|(file, caused_by)| {
            let causes: String = caused_by
                .iter()
                .map(|c| format!("      {}", esc(c)))
                .collect::<Vec<_>>()
                .join(",\n");
            format!(
                "    {{\n      \"file\": {},\n      \"caused_by\": [\n{}\n      ]\n    }}",
                esc(file),
                causes
            )
        })
        .collect::<Vec<_>>()
        .join(",\n");

    format!(
        "{{
  \"commit\": {},
  \"message\": {},
  \"suspects\": [
{}
  ],
  \"iterations\": {},
  \"root_causes\": [
{}
  ],
  \"dependency_chain\": [
{}
  ]
}}",
        esc(h),
        esc(report.message),
        suspects,
        report.iterations,
        roots,
        deps
    )
}
