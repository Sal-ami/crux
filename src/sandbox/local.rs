use std::path::Path;
use std::process::Command;

pub fn run_in_sandbox(cmd: &str, cwd: &Path) -> Option<String> {
    let output = spawn(cmd, cwd, None)?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        None
    }
}

pub fn run_pinned(cmd: &str, cwd: &Path, env: &std::collections::BTreeMap<String, String>) -> String {
    match spawn(cmd, cwd, Some(env)) {
        Some(o) => {
            let mut out = String::from_utf8_lossy(&o.stdout).into_owned();
            out.push_str(&String::from_utf8_lossy(&o.stderr));
            out
        }
        None => String::new(),
    }
}

fn spawn(
    cmd: &str,
    cwd: &Path,
    env: Option<&std::collections::BTreeMap<String, String>>,
) -> Option<std::process::Output> {
    let (shell, args) = crate::shell_cmd();
    let mut c = Command::new(shell);
    c.args(args).arg(cmd).current_dir(cwd);
    if let Some(map) = env {
        c.env_clear();
        for (k, v) in map {
            c.env(k, v);
        }
    }
    c.output().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn runs_command() {
        let dir = TempDir::new().unwrap();
        let r = run_in_sandbox("echo hello", dir.path());
        assert!(r.as_deref().unwrap_or("").contains("hello"));
    }

    #[test]
    fn returns_none_on_failure() {
        let dir = TempDir::new().unwrap();
        let cmd = if cfg!(windows) { "exit /b 1" } else { "false" };
        let r = run_in_sandbox(cmd, dir.path());
        assert!(r.is_none());
    }

    #[test]
    fn pinned_env_is_hermetic() {
        let dir = TempDir::new().unwrap();
        // SAFETY: test process setup
        unsafe { std::env::set_var("CRUX_PIN_MARKER", "outside") };
        let mut env = std::collections::BTreeMap::new();
        env.insert("CRUX_PIN_MARKER".to_string(), "pinned".to_string());
        if cfg!(windows) {
            env.insert("PATH".to_string(), std::env::var("PATH").unwrap_or_default());
        }
        let probe = if cfg!(windows) { "echo %CRUX_PIN_MARKER%" } else { "echo $CRUX_PIN_MARKER" };
        let r = run_pinned(probe, dir.path(), &env);
        assert!(r.contains("pinned"), "got: {r}");
        assert!(!r.contains("outside"));
    }
}
