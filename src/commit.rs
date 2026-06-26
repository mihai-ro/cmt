use crate::types::Config;
use crate::{config, git, picker, ui};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    terminal,
};
use std::io::{self, BufRead, Write};

pub struct Draft {
    pub type_: String,
    pub scope: String,
    pub desc: String,
    pub body: String,
    pub breaking: bool,
    pub breaking_desc: String,
    pub footer: String,
}

pub fn assemble(d: &Draft) -> String {
    let mut header = d.type_.clone();
    if !d.scope.is_empty() {
        header = format!("{}({})", d.type_, d.scope);
    }
    if d.breaking {
        header.push('!');
    }
    header = format!("{}: {}", header, d.desc);

    let mut msg = header;
    if !d.body.is_empty() {
        msg = format!("{}\n\n{}", msg, d.body);
    }
    if d.breaking && !d.breaking_desc.is_empty() {
        msg = format!("{}\n\nBREAKING CHANGE: {}", msg, d.breaking_desc);
    }
    if !d.footer.is_empty() {
        msg = format!("{}\n\n{}", msg, d.footer);
    }
    msg
}

fn prompt_line(p: &ui::Palette, label: &str, hint: &str) -> String {
    let mut e = io::stderr();
    let _ = write!(
        e,
        "\n  {}{}{}  {}{}{}  {}›{} ",
        p.accent_bold, label, p.reset, p.muted, hint, p.reset, p.accent, p.reset
    );
    let _ = e.flush();
    read_line()
}

fn read_line() -> String {
    if terminal::enable_raw_mode().is_err() {
        return read_line_cooked();
    }
    let mut out = io::stderr();
    let mut chars: Vec<char> = Vec::new();
    let mut pos: usize = 0;

    loop {
        let ev = match event::read() {
            Ok(e) => e,
            Err(_) => break,
        };
        let Event::Key(KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            ..
        }) = ev
        else {
            continue;
        };
        if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
            let _ = write!(out, "\r\n");
            let _ = out.flush();
            let _ = terminal::disable_raw_mode();
            std::process::exit(1);
        }
        match code {
            KeyCode::Enter => break,
            KeyCode::Char(c) => {
                chars.insert(pos, c);
                pos += 1;
                if pos == chars.len() {
                    let _ = write!(out, "{c}");
                } else {
                    let tail: String = chars[pos..].iter().collect();
                    let n = tail.chars().count();
                    let _ = write!(out, "{c}{tail}\x1b[K\x1b[{n}D");
                }
                let _ = out.flush();
            }
            KeyCode::Backspace => {
                if pos > 0 {
                    chars.remove(pos - 1);
                    pos -= 1;
                    let tail: String = chars[pos..].iter().collect();
                    let n = tail.chars().count();
                    let _ = write!(out, "\x1b[1D{tail}\x1b[K");
                    if n > 0 {
                        let _ = write!(out, "\x1b[{n}D");
                    }
                    let _ = out.flush();
                }
            }
            KeyCode::Delete => {
                if pos < chars.len() {
                    chars.remove(pos);
                    let tail: String = chars[pos..].iter().collect();
                    let n = tail.chars().count();
                    let _ = write!(out, "{tail}\x1b[K");
                    if n > 0 {
                        let _ = write!(out, "\x1b[{n}D");
                    }
                    let _ = out.flush();
                }
            }
            KeyCode::Left => {
                if pos > 0 {
                    pos -= 1;
                    let _ = write!(out, "\x1b[1D");
                    let _ = out.flush();
                }
            }
            KeyCode::Right => {
                if pos < chars.len() {
                    pos += 1;
                    let _ = write!(out, "\x1b[1C");
                    let _ = out.flush();
                }
            }
            KeyCode::Home => {
                if pos > 0 {
                    let _ = write!(out, "\x1b[{pos}D");
                    let _ = out.flush();
                    pos = 0;
                }
            }
            KeyCode::End => {
                if pos < chars.len() {
                    let n = chars.len() - pos;
                    let _ = write!(out, "\x1b[{n}C");
                    let _ = out.flush();
                    pos = chars.len();
                }
            }
            _ => {}
        }
    }

    let _ = write!(out, "\r\n");
    let _ = out.flush();
    let _ = terminal::disable_raw_mode();
    chars.into_iter().collect()
}

fn read_line_cooked() -> String {
    let mut s = String::new();
    #[cfg(unix)]
    {
        if let Ok(tty) = std::fs::OpenOptions::new().read(true).open("/dev/tty") {
            let _ = io::BufReader::new(tty).read_line(&mut s);
            return s.trim_end_matches(['\n', '\r']).to_string();
        }
    }
    let _ = io::stdin().read_line(&mut s);
    s.trim_end_matches(['\n', '\r']).to_string()
}

fn type_label(t: &crate::types::CommitType, p: &ui::Palette) -> String {
    let badge = match t.semver.as_str() {
        "minor" => format!(
            "  {}[{}{}minor{}{}]{}",
            p.muted, p.reset, p.green, p.reset, p.muted, p.reset
        ),
        "patch" => format!(
            "  {}[{}{}patch{}{}]{}",
            p.muted, p.reset, p.yellow, p.reset, p.muted, p.reset
        ),
        _ => "         ".to_string(),
    };
    format!(
        "{}  {}{:<10}{}{}  {}{}{}",
        t.emoji, p.bold, t.r#type, p.reset, badge, p.muted, t.description, p.reset
    )
}

fn select_type(cfg: &Config, p: &ui::Palette) -> String {
    let labels: Vec<String> = cfg.types.iter().map(|t| type_label(t, p)).collect();
    let idx = picker::select("Select commit type:", &labels).unwrap_or(0);
    cfg.types[idx].r#type.clone()
}

fn select_scope(cfg: &Config, p: &ui::Palette) -> String {
    if !cfg.scopes.is_empty() {
        let mut labels: Vec<String> = cfg
            .scopes
            .iter()
            .map(|s| format!("{}{}{}", p.bold, s, p.reset))
            .collect();
        labels.push(format!(
            "{}─  custom{}  {}type your own{}",
            p.muted, p.reset, p.muted, p.reset
        ));
        labels.push(format!("{}─  skip{}", p.muted, p.reset));
        let idx = picker::select("scope", &labels).unwrap_or(labels.len() - 1);
        let custom_idx = cfg.scopes.len();
        let skip_idx = cfg.scopes.len() + 1;
        if idx < custom_idx {
            return cfg.scopes[idx].clone();
        } else if idx == custom_idx {
            return prompt_line(p, "scope", "enter custom scope");
        } else {
            let _ = skip_idx;
            return String::new();
        }
    }
    prompt_line(p, "scope", "leave blank to omit")
}

fn input_description(cfg: &Config, p: &ui::Palette) -> String {
    let mut e = io::stderr();
    let _ = write!(
        e,
        "\n  {}description{}  {}imperative, present tense{}\n",
        p.accent_bold, p.reset, p.muted, p.reset
    );
    let _ = e.flush();
    loop {
        let _ = write!(e, "  {}›{} ", p.accent, p.reset);
        let _ = e.flush();
        let desc = read_line();
        if desc.is_empty() {
            let _ = writeln!(
                e,
                "  {}⚑{}  {}Description is required{}",
                p.yellow, p.reset, p.yellow, p.reset
            );
            let _ = e.flush();
        } else if desc.chars().count() > cfg.rules.max_header_length {
            let _ = writeln!(
                e,
                "  {}⚑{}  {}Description is {} chars — keep it under {} for readability{}",
                p.yellow,
                p.reset,
                p.yellow,
                desc.chars().count(),
                cfg.rules.max_header_length,
                p.reset
            );
            let _ = write!(
                e,
                "  {}use anyway?{} {}[y/N]{}  {}›{} ",
                p.yellow, p.reset, p.muted, p.reset, p.accent, p.reset
            );
            let _ = e.flush();
            let c = read_line();
            if c.eq_ignore_ascii_case("y") {
                return desc;
            }
        } else {
            return desc;
        }
    }
}

fn input_body(p: &ui::Palette) -> String {
    let mut e = io::stderr();
    let _ = write!(
        e,
        "\n  {}body{}  {}optional — empty line to finish{}\n",
        p.accent_bold, p.reset, p.muted, p.reset
    );
    let _ = e.flush();
    let mut lines = Vec::new();
    loop {
        let _ = write!(e, "  {}›{} ", p.accent, p.reset);
        let _ = e.flush();
        let l = read_line();
        if l.is_empty() {
            break;
        }
        lines.push(l);
    }
    lines.join("\n")
}

fn input_breaking(p: &ui::Palette) -> (bool, String) {
    let mut e = io::stderr();
    loop {
        let _ = write!(
            e,
            "\n  {}breaking change?{}  {}[y/N]{}  {}›{} ",
            p.accent_bold, p.reset, p.muted, p.reset, p.accent, p.reset
        );
        let _ = e.flush();
        match read_line().to_lowercase().as_str() {
            "y" | "yes" => {
                let _ = write!(
                    e,
                    "  {}breaking change{}  {}›{} ",
                    p.red, p.reset, p.accent, p.reset
                );
                let _ = e.flush();
                return (true, read_line());
            }
            "n" | "no" | "" => return (false, String::new()),
            _ => {
                let _ = writeln!(
                    e,
                    "  {}⚑{}  {}Enter y or n{}",
                    p.yellow, p.reset, p.yellow, p.reset
                );
                let _ = e.flush();
            }
        }
    }
}

fn input_footer(p: &ui::Palette) -> String {
    let mut e = io::stderr();
    let _ = write!(
        e,
        "\n  {}footer{}  {}e.g. Closes #42 — leave blank to skip{}\n",
        p.accent_bold, p.reset, p.muted, p.reset
    );
    let _ = write!(e, "  {}›{} ", p.accent, p.reset);
    let _ = e.flush();
    read_line()
}

fn term_cols() -> usize {
    crossterm::terminal::size()
        .map(|(c, _)| c as usize)
        .unwrap_or(80)
}

pub fn build_draft(cfg: &Config, p: &ui::Palette) -> Draft {
    let type_ = select_type(cfg, p);
    let scope = select_scope(cfg, p);
    let desc = input_description(cfg, p);
    let body = input_body(p);
    let (breaking, breaking_desc) = input_breaking(p);
    let footer = input_footer(p);
    Draft {
        type_,
        scope,
        desc,
        body,
        breaking,
        breaking_desc,
        footer,
    }
}

pub fn run(write_only: bool, dry_run: bool) {
    let p = ui::palette();
    if !write_only && !dry_run {
        if !git::in_repo() {
            eprintln!("  {}✖{}  Not inside a git repository", p.red, p.reset);
            std::process::exit(1);
        }
        if !git::has_staged_changes() {
            eprintln!(
                "  {}⚑{}  {}No staged changes detected. Did you forget `git add`?{}",
                p.yellow, p.reset, p.yellow, p.reset
            );
            std::process::exit(0);
        }
    }

    let mut e = io::stderr();
    let _ = write!(
        e,
        "\n  {}cmt{}  {}conventional commits  v{}{}\n",
        p.accent_bold,
        p.reset,
        p.muted,
        env!("CARGO_PKG_VERSION"),
        p.reset
    );
    let _ = writeln!(
        e,
        "  {}─────────────────────────────────────{}",
        p.muted, p.reset
    );
    let _ = e.flush();

    let cfg = config::load();
    let draft = build_draft(&cfg, &p);
    let msg = assemble(&draft);

    if write_only {
        eprint!("{}", ui::commit_box(&msg, &p, term_cols()));
        eprintln!();
        print!("{}", msg);
        return;
    }
    if dry_run {
        eprint!("{}", ui::commit_box(&msg, &p, term_cols()));
        println!("\n{}", msg);
        return;
    }
    confirm_and_commit(&msg, &p);
}

fn confirm_and_commit(msg: &str, p: &ui::Palette) {
    let mut e = io::stderr();
    eprint!("{}", ui::commit_box(msg, p, term_cols()));
    loop {
        let _ = write!(
            e,
            "\n  {}commit?{}  {}[Y/n]{}  {}›{} ",
            p.accent_bold, p.reset, p.muted, p.reset, p.accent, p.reset
        );
        let _ = e.flush();
        match read_line().to_lowercase().as_str() {
            "" | "y" | "yes" => {
                if git::commit(msg, false)
                    .map(|s| s.success())
                    .unwrap_or(false)
                {
                    eprintln!("\n  {}✔{} {}committed{}", p.green, p.reset, p.bold, p.reset);
                } else {
                    eprintln!("\n  {}✖{}  commit failed", p.red, p.reset);
                    std::process::exit(1);
                }
                return;
            }
            "n" | "no" => {
                eprintln!(
                    "  {}⚑{}  {}Commit aborted.{}",
                    p.yellow, p.reset, p.yellow, p.reset
                );
                std::process::exit(0);
            }
            _ => {
                let _ = writeln!(
                    e,
                    "  {}⚑{}  {}Enter y or n{}",
                    p.yellow, p.reset, p.yellow, p.reset
                );
                let _ = e.flush();
            }
        }
    }
}

pub fn edit_in_editor(initial: &str) -> String {
    use std::process::Command;
    let dir = std::env::temp_dir();
    let path = dir.join(format!("cmt-commit-{}.txt", std::process::id()));
    let _ = std::fs::write(&path, initial);
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| {
        if cfg!(windows) {
            "notepad".into()
        } else {
            "vi".into()
        }
    });
    let _ = Command::new(editor).arg(&path).status();
    let edited = std::fs::read_to_string(&path).unwrap_or_else(|_| initial.to_string());
    let _ = std::fs::remove_file(&path);
    edited.trim_end_matches(['\n', '\r']).to_string()
}
