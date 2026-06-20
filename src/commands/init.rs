use crate::{git, hooks, ui};
use std::path::Path;

pub const SCHEMA_URL: &str =
    "https://raw.githubusercontent.com/mihai-ro/cmt/main/schema/cmt.schema.json";

pub fn config_template() -> String {
    format!(
        r#"{{
  "$schema": "{SCHEMA_URL}",
  "customTypes": [
    {{ "type": "wip", "emoji": "🚧", "semver": "none", "description": "Work in progress" }}
  ],
  "scopes": [],
  "rules": {{
    "maxHeaderLength": 72,
    "requireScope": false,
    "allowBreakingChanges": ["feat", "fix"],
    "disallowUpperCaseDescription": false,
    "disallowTrailingPeriod": false
  }}
}}
"#
    )
}

pub fn run(args: &[String]) {
    let p = ui::palette();
    if !git::in_repo() {
        eprintln!(
            "  {}\u{2716}{}  Not inside a git repository",
            p.red, p.reset
        );
        std::process::exit(1);
    }
    let root = git::top_level().unwrap_or_else(|| ".".to_string());
    let root = Path::new(&root);

    let cfg_path = root.join(".cmt.json");
    if !cfg_path.exists() {
        let _ = std::fs::write(&cfg_path, config_template());
        eprintln!("  {}\u{2714}{} Created .cmt.json", p.green, p.reset);
    }

    let use_husky = args.iter().any(|a| a == "--husky");
    let use_lint = args.iter().any(|a| a == "--lint");

    if use_husky {
        let dir = root.join(".husky");
        let _ = std::fs::create_dir_all(&dir);
        let _ = hooks::install_block(
            &dir.join("prepare-commit-msg"),
            &hooks::prepare_snippet(),
            "cmt",
            false,
        );
        eprintln!(
            "  {}\u{2714}{} Created .husky/prepare-commit-msg",
            p.green, p.reset
        );
        if use_lint {
            let _ = hooks::install_block(
                &dir.join("commit-msg"),
                &hooks::lint_snippet(),
                "cmt-lint",
                false,
            );
            eprintln!("  {}\u{2714}{} Created .husky/commit-msg", p.green, p.reset);
        }
        eprintln!(
            "  {}Commit .husky/ hooks to share with your team.{}",
            p.dim, p.reset
        );
    } else {
        let dir = root.join(".git").join("hooks");
        let _ = std::fs::create_dir_all(&dir);
        let _ = hooks::install_block(
            &dir.join("prepare-commit-msg"),
            &hooks::prepare_snippet(),
            "cmt",
            true,
        );
        eprintln!(
            "  {}\u{2714}{} Created .git/hooks/prepare-commit-msg",
            p.green, p.reset
        );
        if use_lint {
            let _ = hooks::install_block(
                &dir.join("commit-msg"),
                &hooks::lint_snippet(),
                "cmt-lint",
                true,
            );
            eprintln!(
                "  {}\u{2714}{} Created .git/hooks/commit-msg",
                p.green, p.reset
            );
        }
    }

    eprintln!("\n  {}\u{2714}{} Done.", p.green, p.reset);
    eprintln!(
        "  {}git commit  or  cmt commit — both work{}\n",
        p.dim, p.reset
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_is_valid_json_and_has_schema() {
        let v: serde_json::Value = serde_json::from_str(&config_template()).unwrap();
        assert!(v.get("$schema").is_some());
        assert_eq!(v["rules"]["maxHeaderLength"], 72);
    }
}
