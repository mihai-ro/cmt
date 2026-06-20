use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_cmt")
}

#[test]
fn version_flag_prints_version() {
    let out = Command::new(bin()).arg("--version").output().unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("cmt version"));
}

#[test]
fn lint_stdin_exit_codes() {
    use std::io::Write;
    let mut child = Command::new(bin())
        .arg("lint")
        .env("NO_COLOR", "1")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"feat: ok").unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());

    let mut child = Command::new(bin())
        .arg("lint")
        .env("NO_COLOR", "1")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"nope").unwrap();
    let out = child.wait_with_output().unwrap();
    assert_eq!(out.status.code(), Some(1));
}
