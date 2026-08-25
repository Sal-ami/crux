use crate::adapter::test;
use std::path::Path;

pub fn ddmin(candidates: &[String], _range: &str, cmd: &str, cwd: &Path) -> (Vec<String>, usize) {
    if candidates.is_empty() {
        return (Vec::new(), 0);
    }
    ddmin_set(candidates, &|subset| {
        let joined = subset.join(" ");
        let result = test::run(&format!("{cmd} {joined}"), cwd);
        result.passed
    })
}

pub fn ddmin_set(candidates: &[String], test_fn: &dyn Fn(&[String]) -> bool) -> (Vec<String>, usize) {
    if candidates.is_empty() {
        return (Vec::new(), 0);
    }
    let mut remaining: Vec<String> = candidates.to_vec();
    let mut iterations = 0;
    let mut changed = true;
    while changed {
        changed = false;
        if remaining.len() <= 1 {
            break;
        }
        let chunk_size = (remaining.len() / 2).max(1);
        let num_chunks = remaining.len().div_ceil(chunk_size);
        for i in 0..num_chunks {
            iterations += 1;
            let subset: Vec<String> = remaining
                .iter()
                .enumerate()
                .filter(|(j, _)| j / chunk_size != i)
                .map(|(_, v)| v.clone())
                .collect();
            if subset.is_empty() {
                continue;
            }
            if test_fn(&subset) {
                remaining = subset;
                changed = true;
                break;
            }
        }
    }
    (remaining, iterations)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ddmin_empty() {
        let (result, iters) = ddmin_set(&[], &|_| false);
        assert!(result.is_empty());
        assert_eq!(iters, 0);
    }

    #[test]
    fn ddmin_single() {
        let input = vec!["a".to_string()];
        let (result, _) = ddmin_set(&input, &|s| s.is_empty());
        assert_eq!(result, vec!["a"]);
    }

    #[test]
    fn ddmin_reduces() {
        let input: Vec<String> = (0..8).map(|i| i.to_string()).collect();
        let (result, iters) = ddmin_set(&input, &|s| s.len() >= 4);
        assert!(result.len() <= 4, "expected <= 4, got {}", result.len());
        assert!(iters > 0);
    }

    #[test]
    fn ddmin_minimal() {
        let input: Vec<String> = (0..4).map(|i| i.to_string()).collect();
        let (result, _) = ddmin_set(&input, &|s| s.len() >= 2);
        assert!(result.len() <= 2, "expected <= 2, got {}", result.len());
    }
}
