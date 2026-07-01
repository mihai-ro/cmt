use std::io::IsTerminal;

pub struct Palette {
    pub reset: &'static str,
    pub bold: &'static str,
    pub dim: &'static str,
    pub accent: &'static str,
    pub accent_bold: &'static str,
    pub green: &'static str,
    pub yellow: &'static str,
    pub red: &'static str,
    pub blue: &'static str,
    pub muted: &'static str,
}

impl Palette {
    pub fn plain() -> Self {
        Palette {
            reset: "",
            bold: "",
            dim: "",
            accent: "",
            accent_bold: "",
            green: "",
            yellow: "",
            red: "",
            blue: "",
            muted: "",
        }
    }
    fn colored() -> Self {
        Palette {
            reset: "\x1b[0m",
            bold: "\x1b[1m",
            dim: "\x1b[2m",
            accent: "\x1b[38;2;86;182;194m",
            accent_bold: "\x1b[1;38;2;86;182;194m",
            green: "\x1b[38;2;152;195;121m",
            yellow: "\x1b[38;2;229;192;123m",
            red: "\x1b[38;2;224;108;117m",
            blue: "\x1b[38;2;97;175;239m",
            muted: "\x1b[38;2;92;99;112m",
        }
    }
}

pub fn palette() -> Palette {
    if std::env::var_os("NO_COLOR").is_some() {
        return Palette::plain();
    }
    if std::io::stderr().is_terminal() {
        Palette::colored()
    } else {
        Palette::plain()
    }
}

/// Read one line from /dev/tty, falling back to stdin. None on EOF.
pub fn read_tty_line() -> Option<String> {
    let mut s = String::new();
    #[cfg(unix)]
    {
        use std::io::BufRead;
        if let Ok(tty) = std::fs::OpenOptions::new().read(true).open("/dev/tty") {
            return match std::io::BufReader::new(tty).read_line(&mut s) {
                Ok(0) | Err(_) => None,
                Ok(_) => Some(s.trim_end_matches(['\n', '\r']).to_string()),
            };
        }
    }
    match std::io::stdin().read_line(&mut s) {
        Ok(0) | Err(_) => None,
        Ok(_) => Some(s.trim_end_matches(['\n', '\r']).to_string()),
    }
}

pub fn wordwrap(text: &str, width: usize) -> Vec<String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return vec![String::new()];
    }
    let mut lines = Vec::new();
    let mut line = String::new();
    for w in words {
        if line.is_empty() {
            line = w.to_string();
        } else if line.chars().count() + 1 + w.chars().count() <= width {
            line.push(' ');
            line.push_str(w);
        } else {
            lines.push(std::mem::take(&mut line));
            line = w.to_string();
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

pub fn commit_box(msg: &str, p: &Palette, term_cols: usize) -> String {
    let max_w = term_cols.saturating_sub(2).max(50);
    let longest = msg.lines().map(|l| l.chars().count()).max().unwrap_or(0);
    let mut w = (longest + 6).max(50);
    if w > max_w {
        w = max_w;
    }
    let content_w = w - 6;
    let bar = "─".repeat(w.saturating_sub(19));
    let mut out = String::new();
    out.push_str(&format!(
        "\n  {}╭─ commit message {}╮{}\n",
        p.muted, bar, p.reset
    ));
    for line in msg.lines() {
        for wrapped in wordwrap(line, content_w) {
            let pad = content_w.saturating_sub(wrapped.chars().count());
            out.push_str(&format!(
                "  {}│{}  {}{}{}{}  {}│{}\n",
                p.muted,
                p.reset,
                p.bold,
                wrapped,
                " ".repeat(pad),
                p.reset,
                p.muted,
                p.reset
            ));
        }
    }
    let bottom = "─".repeat(w.saturating_sub(2));
    out.push_str(&format!("  {}╰{}╯{}\n", p.muted, bottom, p.reset));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wordwrap_breaks_on_width() {
        let out = wordwrap("the quick brown fox", 9);
        assert_eq!(out, vec!["the quick", "brown fox"]);
    }

    #[test]
    fn wordwrap_empty_is_single_blank() {
        assert_eq!(wordwrap("", 10), vec![""]);
    }

    #[test]
    fn no_color_yields_empty_codes() {
        let p = Palette::plain();
        assert_eq!(p.accent, "");
        assert_eq!(p.reset, "");
    }
}
