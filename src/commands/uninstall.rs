use crate::{git, hooks, ui};
use std::path::Path;

pub fn run(_args: &[String]) {
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
    let mut removed = false;

    for dir in [root.join(".git").join("hooks"), root.join(".husky")] {
        if let Ok(o) = hooks::remove_block(&dir.join("prepare-commit-msg"), "cmt") {
            if o != hooks::Outcome::NoOp {
                removed = true;
            }
        }
        if let Ok(o) = hooks::remove_block(&dir.join("commit-msg"), "cmt-lint") {
            if o != hooks::Outcome::NoOp {
                removed = true;
            }
        }
    }

    // legacy "Installed by cmt" hooks
    for path in [
        root.join(".git").join("hooks").join("commit-msg"),
        root.join(".husky").join("commit-msg"),
    ] {
        if path.exists() {
            if let Ok(c) = std::fs::read_to_string(&path) {
                if c.contains("Installed by cmt") {
                    let _ = std::fs::remove_file(&path);
                    removed = true;
                }
            }
        }
    }

    let cfg = root.join(".cmt.json");
    if cfg.exists() {
        let _ = std::fs::remove_file(&cfg);
        eprintln!("  {}\u{2714}{} Removed .cmt.json", p.green, p.reset);
        removed = true;
    }

    if !removed {
        eprintln!("  {}\u{00b7}{} Nothing to remove.", p.blue, p.reset);
    }
    println!();
}
