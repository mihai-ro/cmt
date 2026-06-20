#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitType {
    pub r#type: String,
    pub emoji: String,
    pub semver: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rules {
    pub max_header_length: usize,
    pub require_scope: bool,
    pub disallow_upper_case_description: bool,
    pub disallow_trailing_period: bool,
    pub allow_breaking_changes: Vec<String>,
}

impl Default for Rules {
    fn default() -> Self {
        Rules {
            max_header_length: 72,
            require_scope: false,
            disallow_upper_case_description: false,
            disallow_trailing_period: false,
            allow_breaking_changes: vec!["feat".into(), "fix".into()],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub types: Vec<CommitType>,
    pub scopes: Vec<String>,
    pub rules: Rules,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            types: builtin_types(),
            scopes: Vec::new(),
            rules: Rules::default(),
        }
    }
}

pub fn builtin_types() -> Vec<CommitType> {
    // (type, emoji, semver, description) — order and emoji bytes are load-bearing.
    const RAW: &[(&str, &str, &str, &str)] = &[
        ("feat", "✨", "minor", "A new feature"),
        ("fix", "🐛", "patch", "A bug fix"),
        ("docs", "📚", "none", "Documentation only changes"),
        (
            "style",
            "💅",
            "none",
            "Formatting, missing semi-colons, etc — no logic change",
        ),
        (
            "refactor",
            "♻️ ",
            "none",
            "Code change that neither fixes a bug nor adds a feature",
        ),
        (
            "perf",
            "⚡",
            "patch",
            "A code change that improves performance",
        ),
        ("test", "🧪", "none", "Adding or correcting tests"),
        (
            "build",
            "🏗️ ",
            "none",
            "Changes to build system or external dependencies",
        ),
        (
            "ci",
            "🔧",
            "none",
            "Changes to CI/CD configuration files and scripts",
        ),
        (
            "chore",
            "🔩",
            "none",
            "Other changes that don't modify src or test files",
        ),
        ("revert", "⏪", "patch", "Reverts a previous commit"),
    ];
    RAW.iter()
        .map(|(t, e, s, d)| CommitType {
            r#type: (*t).into(),
            emoji: (*e).into(),
            semver: (*s).into(),
            description: (*d).into(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtins_have_expected_order_and_count() {
        let t = builtin_types();
        assert_eq!(t.len(), 11);
        assert_eq!(t[0].r#type, "feat");
        assert_eq!(t[0].semver, "minor");
        assert_eq!(t.last().unwrap().r#type, "revert");
    }

    #[test]
    fn default_rules_match_spec() {
        let r = Rules::default();
        assert_eq!(r.max_header_length, 72);
        assert!(!r.require_scope);
        assert_eq!(r.allow_breaking_changes, vec!["feat", "fix"]);
    }
}
