use std::process::{Command, ExitStatus, Stdio};

fn run(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).into_owned())
}

pub fn in_repo() -> bool {
    Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn top_level() -> Option<String> {
    run(&["rev-parse", "--show-toplevel"]).map(|s| s.trim_end().to_string())
}

pub fn has_staged_changes() -> bool {
    // `git diff --cached --quiet` exits 1 when there ARE staged changes.
    Command::new("git")
        .args(["diff", "--cached", "--quiet"])
        .status()
        .map(|s| !s.success())
        .unwrap_or(false)
}

pub fn commit(message: &str, amend: bool) -> std::io::Result<ExitStatus> {
    let mut cmd = Command::new("git");
    cmd.arg("commit");
    if amend {
        cmd.args(["--amend", "--no-edit"]);
    }
    cmd.args(["-m", message]).status()
}

pub fn log_lines(limit: usize) -> Vec<String> {
    run(&[
        "log",
        &format!("-{limit}"),
        "--format=%H|%h|%ad|%an|%s",
        "--date=short",
    ])
    .map(|s| s.lines().map(|l| l.to_string()).collect())
    .unwrap_or_default()
}

pub fn last_message() -> Option<String> {
    run(&["log", "-1", "--format=%B"])
}

pub fn status_porcelain() -> Vec<String> {
    run(&["status", "--porcelain"])
        .map(|s| s.lines().map(|l| l.to_string()).collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn detects_repo_when_run_inside_one() {
        // This crate lives in a git repo, so this should be true in CI/dev.
        assert!(in_repo());
    }
}
