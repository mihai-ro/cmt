use crate::{config, git, ui};

pub fn parse_type(subject: &str) -> Option<String> {
    let bytes = subject.as_bytes();
    if bytes.is_empty() || !(bytes[0] as char).is_ascii_lowercase() {
        return None;
    }
    let mut i = 1;
    while i < bytes.len() {
        let c = bytes[i] as char;
        if c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-' {
            i += 1;
        } else {
            break;
        }
    }
    if i < bytes.len() && matches!(bytes[i], b'(' | b':' | b'!') {
        Some(subject[..i].to_string())
    } else {
        None
    }
}

pub fn is_breaking(subject: &str) -> bool {
    subject.contains("!:")
}

pub fn run(args: &[String]) {
    let p = ui::palette();
    if !git::in_repo() {
        eprintln!("  {}✖{}  Not inside a git repository", p.red, p.reset);
        std::process::exit(1);
    }
    let cfg = config::load();

    let mut limit = 20usize;
    let mut filter_type = String::new();
    let mut filter_author = String::new();
    for a in args {
        if let Some(t) = a.strip_prefix("--type=") {
            filter_type = t.to_string();
        } else if let Some(au) = a.strip_prefix("--author=") {
            filter_author = au.to_string();
        } else if let Ok(n) = a.parse::<usize>() {
            limit = n;
        }
    }

    let mut label = format!("last {limit} commits");
    if !filter_type.is_empty() {
        label.push_str(&format!("  {}type={}{}", p.muted, filter_type, p.reset));
    }
    if !filter_author.is_empty() {
        label.push_str(&format!("  {}author={}{}", p.muted, filter_author, p.reset));
    }
    println!(
        "\n  {}log{}  {}{}{}",
        p.accent_bold, p.reset, p.muted, label, p.reset
    );

    for line in git::log_lines(limit) {
        let parts: Vec<&str> = line.splitn(5, '|').collect();
        if parts.len() < 5 {
            continue;
        }
        let (_hash, short, date, author, subject) =
            (parts[0], parts[1], parts[2], parts[3], parts[4]);
        let ty = parse_type(subject).unwrap_or_default();
        if !filter_type.is_empty() && ty != filter_type {
            continue;
        }
        if !filter_author.is_empty() && !author.contains(&filter_author) {
            continue;
        }

        let mut emoji = "·".to_string();
        let mut color = p.reset;
        if !ty.is_empty() {
            if let Some(t) = cfg.types.iter().find(|t| t.r#type == ty) {
                emoji = t.emoji.clone();
            }
            color = match ty.as_str() {
                "feat" => p.green,
                "fix" | "perf" => p.yellow,
                "revert" => p.red,
                _ => p.muted,
            };
        }
        let breaking = if is_breaking(subject) {
            format!(" {}breaking{}", p.red, p.reset)
        } else {
            String::new()
        };
        println!(
            "  {}{}{}  {}  {}{}{}{}{}",
            p.muted, short, p.reset, emoji, color, p.bold, subject, p.reset, breaking
        );
        println!("  {}{}  {}{}\n", p.muted, date, author, p.reset);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_type_from_subject() {
        assert_eq!(parse_type("feat(api): x"), Some("feat".to_string()));
        assert_eq!(parse_type("fix!: y"), Some("fix".to_string()));
        assert_eq!(parse_type("no type here"), None);
    }

    #[test]
    fn detects_breaking_marker() {
        assert!(is_breaking("feat!: drop"));
        assert!(is_breaking("feat(x)!: drop"));
        assert!(!is_breaking("feat: keep"));
    }
}
