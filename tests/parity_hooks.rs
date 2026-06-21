use cmt::hooks;
use std::fs;
use std::path::Path;

#[test]
fn fresh_install_matches_fixture() {
    let tmp = std::env::temp_dir().join(format!("cmt-hooktest-{}", std::process::id()));
    fs::create_dir_all(&tmp).unwrap();
    let path = tmp.join("prepare-commit-msg");

    hooks::install_block(&path, &hooks::prepare_snippet(), "cmt", true).unwrap();
    let got = fs::read_to_string(&path).unwrap();
    let expected =
        fs::read_to_string(Path::new("tests/fixtures/hooks/prepare_fresh.after")).unwrap();
    assert_eq!(got, expected);

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn fresh_lint_install_matches_fixture() {
    let tmp = std::env::temp_dir().join(format!("cmt-hooktest2-{}", std::process::id()));
    fs::create_dir_all(&tmp).unwrap();
    let path = tmp.join("commit-msg");
    hooks::install_block(&path, &hooks::lint_snippet(), "cmt-lint", true).unwrap();
    let got = fs::read_to_string(&path).unwrap();
    let expected =
        fs::read_to_string(Path::new("tests/fixtures/hooks/commitmsg_fresh.after")).unwrap();
    assert_eq!(got, expected);
    let _ = fs::remove_dir_all(&tmp);
}
