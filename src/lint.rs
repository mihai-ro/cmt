use crate::types::Config;
use crate::ui::Palette;

pub struct LintResult {
    pub header: String,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl LintResult {
    pub fn exit_code(&self) -> i32 {
        if self.errors.is_empty() {
            0
        } else {
            1
        }
    }
}

fn is_valid_type_char(c: char) -> bool {
    c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-'
}

// Parse "type(scope)?!?: desc". Returns (type, has_scope, desc) when the header
// matches the required shape; None otherwise.
fn parse_header(header: &str) -> Option<(String, bool, String)> {
    let bytes = header.as_bytes();
    let mut i = 0;
    // type: starts with [a-z], then [a-z0-9_-]*
    if i >= bytes.len() || !(bytes[i] as char).is_ascii_lowercase() {
        return None;
    }
    let start = i;
    i += 1;
    while i < bytes.len() && is_valid_type_char(bytes[i] as char) {
        i += 1;
    }
    let ty = header[start..i].to_string();
    // optional (scope)
    let mut has_scope = false;
    if i < bytes.len() && bytes[i] == b'(' {
        has_scope = true;
        i += 1;
        while i < bytes.len() && bytes[i] != b')' {
            i += 1;
        }
        if i >= bytes.len() {
            return None; // unterminated scope
        }
        i += 1; // consume ')'
    }
    // optional '!'
    if i < bytes.len() && bytes[i] == b'!' {
        i += 1;
    }
    // ': ' then non-empty rest (at least one char)
    if i >= bytes.len() || bytes[i] != b':' {
        return None;
    }
    i += 1;
    if i >= bytes.len() || bytes[i] != b' ' {
        return None;
    }
    i += 1;
    let desc = header[i..].to_string();
    if desc.is_empty() {
        return None;
    }
    Some((ty, has_scope, desc))
}

pub fn check(msg: &str, cfg: &Config) -> LintResult {
    let header = msg.split('\n').next().unwrap_or("").to_string();
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    match parse_header(&header) {
        None => {
            errors.push("Header must match: type(scope)?: description".to_string());
        }
        Some((ty, has_scope, desc)) => {
            // valid type
            let valid = cfg.types.iter().any(|t| t.r#type == ty);
            if !valid {
                let names: Vec<&str> = cfg.types.iter().map(|t| t.r#type.as_str()).collect();
                errors.push(format!("Unknown type '{}'. Valid: {}", ty, names.join(",")));
            }
            // non-empty desc (parse_header already guarantees non-empty, but keep for parity)
            if desc.is_empty() {
                errors.push("Description must not be empty".to_string());
            }
            // capitalization
            if desc
                .chars()
                .next()
                .map(|c| c.is_ascii_uppercase())
                .unwrap_or(false)
            {
                if cfg.rules.disallow_upper_case_description {
                    errors.push("Description must start with a lowercase letter".to_string());
                } else {
                    warnings
                        .push("Description starts with uppercase — prefer lowercase".to_string());
                }
            }
            // trailing period
            if desc.ends_with('.') {
                if cfg.rules.disallow_trailing_period {
                    errors.push("Description must not end with a period".to_string());
                } else {
                    warnings.push("Description ends with a period — omit it".to_string());
                }
            }
            // header length
            let hlen = header.chars().count();
            if hlen > cfg.rules.max_header_length {
                warnings.push(format!(
                    "Header is {} chars — aim for ≤{}",
                    hlen, cfg.rules.max_header_length
                ));
            }
            // require scope
            if cfg.rules.require_scope && !has_scope {
                errors.push("Scope is required — e.g. feat(auth): ...".to_string());
            }
        }
    }

    // blank line between header and body
    if let Some(rest) = msg.split_once('\n').map(|(_, r)| r) {
        let second = rest.split('\n').next().unwrap_or("");
        if !second.is_empty() {
            errors.push("Blank line required between header and body".to_string());
        }
    }

    LintResult {
        header,
        errors,
        warnings,
    }
}

pub fn render(r: &LintResult, p: &Palette) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "\n  {}lint{}  {}{}{}\n",
        p.accent_bold, p.reset, p.muted, r.header, p.reset
    ));
    for e in &r.errors {
        out.push_str(&format!("  {}✖{}  {}\n", p.red, p.reset, e));
    }
    for w in &r.warnings {
        out.push_str(&format!(
            "  {}⚑{}  {}{}{}\n",
            p.yellow, p.reset, p.yellow, w, p.reset
        ));
    }
    if r.errors.is_empty() && r.warnings.is_empty() {
        out.push_str(&format!(
            "  {}✔{} valid conventional commit\n",
            p.green, p.reset
        ));
    } else if r.errors.is_empty() {
        out.push_str(&format!(
            "\n  {}✔{} valid — with {} suggestion(s)\n",
            p.green,
            p.reset,
            r.warnings.len()
        ));
    }
    out.push('\n');
    out
}
