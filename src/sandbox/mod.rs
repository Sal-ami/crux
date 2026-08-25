pub mod local;

use std::path::Path;

pub fn replay(cmd: &str, cwd: &Path) -> String {
    local::run_in_sandbox(cmd, cwd).unwrap_or_default()
}

pub fn replay_pinned(
    cmd: &str,
    cwd: &Path,
    env: &std::collections::BTreeMap<String, String>,
) -> String {
    local::run_pinned(cmd, cwd, env)
}
