use crate::git;
use crate::slice;
use std::path::Path;

pub fn doctor(cwd: &Path) {
    let mut failures = 0;
    let mut total = 0;

    total += 1;
    if check_git(cwd) {
        eprintln!("[ok] git repository");
    } else {
        eprintln!("[fail] not a git repository");
        failures += 1;
    }

    total += 1;
    if check_commits(cwd) {
        eprintln!("[ok] commit history readable");
    } else {
        eprintln!("[fail] cannot read commit history");
        failures += 1;
    }

    total += 1;
    if check_test_runner(cwd) {
        eprintln!("[ok] test runner found");
    } else {
        eprintln!("[warn] no test runner detected");
    }

    total += 1;
    if check_slicing(cwd) {
        eprintln!("[ok] slicing works");
    } else {
        eprintln!("[warn] slicing returned no results");
    }

    eprintln!();
    eprintln!("{}/{} checks passed", total - failures, total);
    if failures > 0 {
        std::process::exit(1);
    }
}

fn check_git(cwd: &Path) -> bool {
    std::process::Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(cwd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn check_commits(cwd: &Path) -> bool {
    git::log(&git::full_range(cwd), cwd)
        .map(|c| !c.is_empty())
        .unwrap_or(false)
}

fn check_test_runner(cwd: &Path) -> bool {
    let indicators = [
        "Cargo.toml", "pyproject.toml", "go.mod",
        "package.json", "Makefile", "CMakeLists.txt",
    ];
    indicators.iter().any(|f| cwd.join(f).exists())
}

fn check_slicing(cwd: &Path) -> bool {
    let files = std::fs::read_dir(cwd)
        .ok()
        .map(|entries| {
            entries
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.is_file() && p.extension().is_some_and(|e| e == "rs"))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if let Some(file) = files.first() {
        let suspects = slice::slice(&[file.to_string_lossy().to_string()], cwd);
        !suspects.is_empty()
    } else {
        true
    }
}
