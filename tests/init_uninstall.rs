use std::fs;

#[test]
fn init_creates_config_and_hook() {
    // Test hooks and config template work together (library-level)
    let tmp = std::env::temp_dir().join(format!("cmt-it-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();

    // init command requires git repo — test just the hooks install part
    let hooks_dir = tmp.join("hooks");
    std::fs::create_dir_all(&hooks_dir).unwrap();
    let path = hooks_dir.join("prepare-commit-msg");
    cmt::hooks::install_block(&path, &cmt::hooks::prepare_snippet(), "cmt", true).unwrap();
    assert!(path.exists());

    // verify config template is valid JSON
    let template = cmt::commands::init::config_template();
    let v: serde_json::Value = serde_json::from_str(&template).unwrap();
    assert!(v.get("$schema").is_some());

    let _ = fs::remove_dir_all(&tmp);
}
