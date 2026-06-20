use crate::{git, ui};

pub fn run(_args: &[String]) {
    let p = ui::palette();
    if !git::in_repo() {
        eprintln!("  {}✖{}  Not inside a git repository", p.red, p.reset);
        std::process::exit(1);
    }
    println!("\n  {}staged{}\n", p.accent_bold, p.reset);
    let mut staged = 0;
    for line in git::status_porcelain() {
        if line.len() < 4 {
            continue;
        }
        let x = &line[0..1];
        let file = &line[3..];
        let (glyph, color, word) = match x {
            "A" => ("+", p.green, "added"),
            "M" => ("~", p.blue, "modified"),
            "D" => ("-", p.red, "deleted"),
            "R" => ("»", p.yellow, "renamed"),
            "C" => ("»", p.yellow, "copied"),
            _ => continue,
        };
        println!(
            "  {}{}{}  {:<40}  {}{}{}",
            color, glyph, p.reset, file, p.muted, word, p.reset
        );
        staged += 1;
    }
    if staged == 0 {
        eprintln!(
            "  {}⚑{}  {}Nothing staged. Run: git add <file>{}",
            p.yellow, p.reset, p.yellow, p.reset
        );
    }
    println!();
}
