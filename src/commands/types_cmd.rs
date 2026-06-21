use crate::{config, ui};

pub fn run(_args: &[String]) {
    let p = ui::palette();
    let cfg = config::load();
    println!("\n  {}commit types{}", p.accent_bold, p.reset);
    for t in &cfg.types {
        let badge = match t.semver.as_str() {
            "minor" => format!("  {}minor{}", p.green, p.reset),
            "patch" => format!("  {}patch{}", p.yellow, p.reset),
            _ => String::new(),
        };
        println!(
            "  {}  {}{:<12}{}{}  {}{}{}",
            t.emoji, p.bold, t.r#type, p.reset, badge, p.muted, t.description, p.reset
        );
    }
    println!();
    if std::path::Path::new(&config::config_path()).exists() {
        println!(
            "  {}custom types from .cmt.json included above{}\n",
            p.muted, p.reset
        );
    } else {
        println!(
            "  {}run cmt init to configure custom types{}\n",
            p.muted, p.reset
        );
    }
}
