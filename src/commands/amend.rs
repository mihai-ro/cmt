use crate::{commit, config, git, ui};
use std::io::{self, Write};

fn read_line() -> String {
    let mut s = String::new();
    let _ = io::stdin().read_line(&mut s);
    s.trim_end_matches(['\n', '\r']).to_string()
}

pub fn run(_args: &[String]) {
    let p = ui::palette();
    if !git::in_repo() {
        eprintln!("  {}✖{}  Not inside a git repository", p.red, p.reset);
        std::process::exit(1);
    }
    let last = match git::last_message() {
        Some(m) => m.trim_end_matches('\n').to_string(),
        None => {
            eprintln!("  {}✖{}  No commits yet", p.red, p.reset);
            std::process::exit(1);
        }
    };
    let cols = crossterm::terminal::size()
        .map(|(c, _)| c as usize)
        .unwrap_or(80);
    eprint!("{}", ui::commit_box(&last, &p, cols));

    let mut e = io::stderr();
    let _ = write!(
        e,
        "\n  {}amend{}  {}[Y=rebuild / e=edit raw / N=abort]{}  {}›{} ",
        p.accent_bold, p.reset, p.muted, p.reset, p.accent, p.reset
    );
    let _ = e.flush();
    let ans = read_line();

    match ans.as_str() {
        "e" | "E" => {
            let edited = commit::edit_in_editor(&last);
            if git::commit(&edited, true)
                .map(|s| s.success())
                .unwrap_or(false)
            {
                eprintln!("\n  {}✔{} {}amended{}", p.green, p.reset, p.bold, p.reset);
            } else {
                eprintln!("\n  {}✖{}  amend failed", p.red, p.reset);
                std::process::exit(1);
            }
        }
        "" | "y" | "Y" => {
            let cfg = config::load();
            let draft = commit::build_draft(&cfg, &p);
            let msg = commit::assemble(&draft);
            eprint!("{}", ui::commit_box(&msg, &p, cols));
            let _ = write!(
                e,
                "\n  {}commit --amend?{}  {}[Y/n]{}  {}›{} ",
                p.accent_bold, p.reset, p.muted, p.reset, p.accent, p.reset
            );
            let _ = e.flush();
            let c = read_line();
            if c.eq_ignore_ascii_case("n") {
                eprintln!(
                    "  {}⚑{}  {}Amend aborted.{}",
                    p.yellow, p.reset, p.yellow, p.reset
                );
                std::process::exit(0);
            }
            if git::commit(&msg, true)
                .map(|s| s.success())
                .unwrap_or(false)
            {
                eprintln!("\n  {}✔{} {}amended{}", p.green, p.reset, p.bold, p.reset);
            } else {
                eprintln!("\n  {}✖{}  amend failed", p.red, p.reset);
                std::process::exit(1);
            }
        }
        _ => {
            eprintln!(
                "  {}⚑{}  {}Amend aborted.{}",
                p.yellow, p.reset, p.yellow, p.reset
            );
            std::process::exit(0);
        }
    }
}
