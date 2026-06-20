use crate::types::{CommitType, Config, Rules, builtin_types};
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize, Default)]
#[serde(default)]
struct RawType {
    r#type: String,
    emoji: Option<String>,
    semver: Option<String>,
    description: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct RawRules {
    max_header_length: Option<usize>,
    require_scope: Option<bool>,
    disallow_upper_case_description: Option<bool>,
    disallow_trailing_period: Option<bool>,
    allow_breaking_changes: Option<Vec<String>>,
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct RawConfig {
    custom_types: Vec<RawType>,
    scopes: Vec<String>,
    rules: RawRules,
}

pub fn config_path() -> String {
    std::env::var("CMT_CONFIG_FILE").unwrap_or_else(|_| ".cmt.json".to_string())
}

pub fn load() -> Config {
    load_from(Path::new(&config_path()))
}

pub fn load_from(path: &Path) -> Config {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(_) => return Config::default(),
    };
    let raw: RawConfig = match serde_json::from_str(&text) {
        Ok(r) => r,
        Err(_) => return Config::default(),
    };

    let mut types = builtin_types();
    for rt in raw.custom_types {
        if rt.r#type.is_empty() {
            continue;
        }
        types.push(CommitType {
            r#type: rt.r#type,
            emoji: rt.emoji.unwrap_or_else(|| "⚙️ ".to_string()),
            semver: rt.semver.unwrap_or_else(|| "none".to_string()),
            description: rt.description.unwrap_or_else(|| "Custom type".to_string()),
        });
    }

    let d = Rules::default();
    let mut allow = raw
        .rules
        .allow_breaking_changes
        .unwrap_or_else(|| d.allow_breaking_changes.clone());
    if allow.is_empty() {
        allow = d.allow_breaking_changes.clone();
    }
    let rules = Rules {
        max_header_length: raw.rules.max_header_length.unwrap_or(d.max_header_length),
        require_scope: raw.rules.require_scope.unwrap_or(d.require_scope),
        disallow_upper_case_description: raw
            .rules
            .disallow_upper_case_description
            .unwrap_or(d.disallow_upper_case_description),
        disallow_trailing_period: raw
            .rules
            .disallow_trailing_period
            .unwrap_or(d.disallow_trailing_period),
        allow_breaking_changes: allow,
    };

    Config {
        types,
        scopes: raw.scopes,
        rules,
    }
}

pub fn dump_canonical(c: &Config) -> String {
    let type_names: Vec<&str> = c.types.iter().map(|t| t.r#type.as_str()).collect();
    format!(
        "maxHeaderLength={}\nrequireScope={}\ndisallowUpperCaseDescription={}\ndisallowTrailingPeriod={}\nallowBreakingChanges={}\nscopes={}\ntypes={}\n",
        c.rules.max_header_length,
        c.rules.require_scope,
        c.rules.disallow_upper_case_description,
        c.rules.disallow_trailing_period,
        c.rules.allow_breaking_changes.join(","),
        c.scopes.join(","),
        type_names.join(","),
    )
}
