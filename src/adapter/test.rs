use std::path::Path;
use std::process::Command;

pub struct TestResult {
    pub passed: bool,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub fn run(cmd: &str, cwd: &Path) -> TestResult {
    let (shell, args) = crate::shell_cmd();
    let output = Command::new(shell)
        .args(args)
        .arg(cmd)
        .current_dir(cwd)
        .output();
    match output {
        Ok(o) => TestResult {
            passed: o.status.success(),
            exit_code: o.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&o.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&o.stderr).into_owned(),
        },
        Err(_) => TestResult {
            passed: false,
            exit_code: -1,
            stdout: String::new(),
            stderr: "failed to execute command".into(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn passes_on_true() {
        let dir = TempDir::new().unwrap();
        let cmd = if cfg!(windows) { "exit /b 0" } else { "true" };
        let r = run(cmd, dir.path());
        assert!(r.passed);
    }

    #[test]
    fn fails_on_false() {
        let dir = TempDir::new().unwrap();
        let cmd = if cfg!(windows) { "exit /b 1" } else { "false" };
        let r = run(cmd, dir.path());
        assert!(!r.passed);
    }

    #[test]
    fn captures_stdout() {
        let dir = TempDir::new().unwrap();
        let r = run("echo hello", dir.path());
        assert!(r.stdout.contains("hello"));
    }

    #[test]
    fn nonexistent_command() {
        let dir = TempDir::new().unwrap();
        let r = run("nonexistent_cmd_xyz_12345", dir.path());
        assert!(!r.passed);
    }
}
