pub struct Hunk {
    pub start: usize,
    pub lines: Vec<HunkLine>,
}

pub struct HunkLine {
    pub prefix: char,
    pub content: String,
}

pub fn parse_hunks(diff: &str) -> Vec<Hunk> {
    let mut hunks = Vec::new();
    let mut current: Option<Hunk> = None;
    for line in diff.lines() {
        if line.starts_with("@@") {
            if let Some(h) = current.take() {
                hunks.push(h);
            }
            let start = parse_hunk_start(line);
            current = Some(Hunk {
                start,
                lines: Vec::new(),
            });
        } else if let Some(ref mut h) = current
            && (line.starts_with('+') || line.starts_with('-') || line.starts_with(' '))
        {
            let prefix = line.chars().next().unwrap_or(' ');
            let content = if line.len() > 1 { line[1..].to_string() } else { String::new() };
            h.lines.push(HunkLine { prefix, content });
        }
    }
    if let Some(h) = current {
        hunks.push(h);
    }
    hunks
}

fn parse_hunk_start(header: &str) -> usize {
    for part in header.split(' ') {
        if let Some(rest) = part.strip_prefix('+')
            && let Some(num) = rest.split(',').next()
            && let Ok(n) = num.parse()
        {
            return n;
        }
    }
    0
}

pub fn render_hunks(hunks: &[Hunk]) -> String {
    let mut out = String::new();
    for hunk in hunks {
        out.push_str(&format!("@@ +{} @@\n", hunk.start));
        for line in &hunk.lines {
            out.push_str(&format!("{}{}\n", line.prefix, line.content));
        }
    }
    out
}

#[derive(Clone)]
pub struct Piece {
    pub header: Vec<String>,
    pub body: Vec<String>,
}

impl Piece {
    pub fn render(&self) -> String {
        let mut out = self.header.join("\n");
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(&self.body.join("\n"));
        out.push('\n');
        out
    }
}

pub fn parse_pieces(diff: &str) -> Vec<Piece> {
    let mut pieces = Vec::new();
    let mut header: Vec<String> = Vec::new();
    let mut current: Option<Piece> = None;
    for line in diff.lines() {
        if line.starts_with("diff --git") {
            if let Some(p) = current.take() {
                pieces.push(p);
            }
            header.clear();
            header.push(line.to_string());
        } else if line.starts_with("@@") {
            if let Some(p) = current.take() {
                pieces.push(p);
            }
            current = Some(Piece {
                header: header.clone(),
                body: vec![line.to_string()],
            });
        } else if let Some(ref mut p) = current {
            p.body.push(line.to_string());
        } else if !header.is_empty() {
            header.push(line.to_string());
        }
    }
    if let Some(p) = current {
        pieces.push(p);
    }
    pieces
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hunks() {
        let diff = "@@ -1,3 +1,4 @@\n line1\n+added\n line2\n-removed\n line3";
        let hunks = parse_hunks(diff);
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].lines.len(), 5);
    }

    #[test]
    fn multiple_hunks() {
        let diff = "@@ -1,2 +1,2 @@\n-a\n+b\n@@ -10,2 +10,2 @@\n-c\n+d";
        let hunks = parse_hunks(diff);
        assert_eq!(hunks.len(), 2);
    }

    #[test]
    fn empty_diff() {
        let hunks = parse_hunks("");
        assert!(hunks.is_empty());
    }

    #[test]
    fn render_roundtrip() {
        let diff = "@@ -1,3 +1,4 @@\n line1\n+added\n line2";
        let hunks = parse_hunks(diff);
        let rendered = render_hunks(&hunks);
        assert!(rendered.contains("@@ +1 @@"));
        assert!(rendered.contains("+added"));
    }

    const SAMPLE: &str = "diff --git a/f.txt b/f.txt\nindex 111..222 100644\n--- a/f.txt\n+++ b/f.txt\n@@ -1,2 +1,2 @@\n-a\n+b\n@@ -5,1 +5,1 @@\n-x\n+y\ndiff --git a/g.txt b/g.txt\nindex 333..444 100644\n--- a/g.txt\n+++ b/g.txt\n@@ -1,1 +1,1 @@\n-p\n+q";

    #[test]
    fn pieces_split_per_hunk_with_headers() {
        let pieces = parse_pieces(SAMPLE);
        assert_eq!(pieces.len(), 3);
        assert!(pieces[0].header.iter().any(|l| l.contains("f.txt")));
        assert!(pieces[2].body[0].starts_with("@@ -1,1"));
        let r = pieces[2].render();
        assert!(r.starts_with("diff --git a/g.txt"));
        assert!(r.contains("-p\n+q"));
    }
}
