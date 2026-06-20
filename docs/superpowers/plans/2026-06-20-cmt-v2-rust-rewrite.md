# cmt v2 Native Rust Rewrite — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the bash `cmt` script with a single native Rust binary that is a 1:1 drop-in replacement, runs natively on Linux/macOS/Windows, and makes wizard arrow-navigation work everywhere with zero exceptions.

**Architecture:** One small Rust binary, no async runtime. Pure-logic modules (config, lint, assemble, picker state machine) are unit-tested in isolation; IO/render layers (crossterm picker, git calls, hook file mutation) wrap them. Parity is locked by fixture snapshots captured once from the old bash script before it is deleted.

**Tech Stack:** Rust (stable), crossterm (TTY + arrow nav), serde + serde_json (config). No clap, no ratatui. Distribution: GitHub Releases binaries, curl/PowerShell installers, npm optionalDependencies wrapper, Homebrew formula.

## Global Constraints

- Binary name: `cmt`. Crate name: `cmt`. npm package: `@mihairo/cmt`. First release version: **2.0.0**.
- Target version string in `--version` / help: read from `CARGO_PKG_VERSION` (single source of truth = `Cargo.toml`).
- Zero behavioral drift from the current bash script — CLI, flags, config, lint, hooks, and stdout/stderr output must match captured fixtures exactly.
- Supported targets: `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `x86_64-apple-darwin`, `aarch64-apple-darwin`, `x86_64-pc-windows-msvc`, `aarch64-pc-windows-msvc`.
- Dependencies limited to: `crossterm`, `serde`, `serde_json` (and `unicode-width` only if box alignment requires it — decide in Task 5, do not add speculatively).
- Config file default: `.cmt.json`, overridable by env `CMT_CONFIG_FILE`.
- Schema URL constant: `https://raw.githubusercontent.com/mihai-ro/cmt/main/schema/cmt.schema.json`. Ship `schema/cmt.schema.json` unchanged.
- Color palette (truecolor): accent `38;2;86;182;194`, green `38;2;152;195;121`, yellow `38;2;229;192;123`, red `38;2;224;108;117`, blue `38;2;97;175;239`, muted `38;2;92;99;112`, plus `\033[1m` bold, `\033[2m` dim, `\033[0m` reset. Honor `NO_COLOR`; emit no color when stdout is not a TTY.
- Builtin commit types (order matters), each `type | emoji | semver | description`:
  - `feat | ✨ | minor | A new feature`
  - `fix | 🐛 | patch | A bug fix`
  - `docs | 📚 | none | Documentation only changes`
  - `style | 💅 | none | Formatting, missing semi-colons, etc — no logic change`
  - `refactor | ♻️  | none | Code change that neither fixes a bug nor adds a feature`
  - `perf | ⚡ | patch | A code change that improves performance`
  - `test | 🧪 | none | Adding or correcting tests`
  - `build | 🏗️  | none | Changes to build system or external dependencies`
  - `ci | 🔧 | none | Changes to CI/CD configuration files and scripts`
  - `chore | 🔩 | none | Other changes that don't modify src or test files`
  - `revert | ⏪ | patch | Reverts a previous commit`
  - (Note: `refactor` and `build` emoji are followed by a trailing space in the source — preserve byte-for-byte.)
- Commands + aliases: `commit|c`, `amend|a`, `lint|l`, `init`, `log`, `status|s`, `types|t`, `completions`, `uninstall`, `version|-v|--version`, `help|-h|--help`. Default command when none given: `commit`. Unknown command falls through to `help`.

---

## File Structure

```
Cargo.toml
src/
  main.rs        # arg dispatch -> command functions; default=commit, unknown=help
  types.rs       # CommitType, Rules, Config; BUILTIN_TYPES
  config.rs      # load Config from .cmt.json (serde_json) merged over defaults
  ui.rs          # Palette (color/NO_COLOR/tty), wordwrap, commit box, msg helpers
  git.rs         # git subprocess wrappers
  lint.rs        # LintResult + check(); 8 rules
  picker/
    state.rs     # PickerState: pure filter/move/scroll state machine (unit-tested)
    mod.rs       # crossterm render + key loop + numbered fallback
  commit.rs      # wizard prompts, assemble_message, confirm/commit
  hooks.rs       # snippets, marker-block install/replace/remove, husky paths
  commands/
    init.rs      # cmd init
    uninstall.rs # cmd uninstall
    log.rs       # cmd log (+ --type/--author filters)
    types_cmd.rs # cmd types
    status.rs    # cmd status
    amend.rs     # cmd amend
    completions.rs # bash/zsh/fish strings
    help.rs      # help text + version
tests/
  fixtures/      # captured parity snapshots (committed)
  parity_lint.rs
  parity_assemble.rs
  parity_config.rs
  parity_hooks.rs
.github/workflows/
  ci.yml         # build+test matrix (ubuntu/macos/windows)
  release.yml    # cross-compile + attach binaries on tag
npm/             # npm wrapper package + per-platform optional packages
install.sh       # rewritten: detect + download release binary
install.ps1      # new: Windows installer
schema/cmt.schema.json  # unchanged
```

---

## Task 1: Scaffold Rust project + CI matrix

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `.github/workflows/ci.yml`
- Create: `rust-toolchain.toml`

**Interfaces:**
- Consumes: nothing.
- Produces: a compiling binary that prints nothing meaningful yet but builds on all 3 OSes; `cargo test` runs.

- [ ] **Step 1: Write `Cargo.toml`**

```toml
[package]
name = "cmt"
version = "2.0.0"
edition = "2021"
description = "Conventional Commits CLI — a single native binary, zero runtime deps."
license = "MIT"
repository = "https://github.com/mihai-ro/cmt"
homepage = "https://github.com/mihai-ro/cmt"
keywords = ["conventional-commits", "commit", "git", "lint", "cli"]
categories = ["command-line-utilities", "development-tools"]

[[bin]]
name = "cmt"
path = "src/main.rs"

[dependencies]
crossterm = "0.28"
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[profile.release]
opt-level = "z"
lto = true
strip = true
codegen-units = 1
panic = "abort"
```

- [ ] **Step 2: Write a minimal `src/main.rs`**

```rust
fn main() {
    // Replaced in Task 15 with full dispatch.
    println!("cmt {}", env!("CARGO_PKG_VERSION"));
}
```

- [ ] **Step 3: Write `rust-toolchain.toml`**

```toml
[toolchain]
channel = "stable"
```

- [ ] **Step 4: Verify it builds and runs**

Run: `cargo run -- ` then `cargo test`
Expected: prints `cmt 2.0.0`; `cargo test` reports `0 tests` and exits 0.

- [ ] **Step 5: Write `.github/workflows/ci.yml`**

```yaml
name: CI
on:
  push:
    branches: [main]
  pull_request:
jobs:
  test:
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
      - uses: Swatinem/rust-cache@v2
      - run: cargo fmt --check
      - run: cargo clippy --all-targets -- -D warnings
      - run: cargo build --verbose
      - run: cargo test --verbose
```

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock src/main.rs rust-toolchain.toml .github/workflows/ci.yml
git commit -m "chore: scaffold rust project and CI matrix"
```

---

## Task 2: Capture parity fixtures, then remove the bash script

This task uses the existing bash `cmt` ONE LAST TIME to record its outputs as committed expected files, then deletes the script and the bash test suite. Everything after this validates against these fixtures.

**Files:**
- Create: `tests/fixtures/lint/*.{msg,out,code}`
- Create: `tests/fixtures/assemble/*.{in,out}`
- Create: `tests/fixtures/config/*.{json,resolved}`
- Create: `tests/fixtures/hooks/*` (before/after hook file contents)
- Delete: `cmt`, `test/helpers.sh`, `test/run_tests.sh`, `test/test_assemble.sh`, `test/test_config.sh`, `test/test_lint.sh`
- Create: `scripts/capture_fixtures.sh` (the one-time recorder; kept for provenance)

**Interfaces:**
- Consumes: nothing.
- Produces: committed fixture files later tasks assert against. Fixture file formats:
  - lint: `NAME.msg` (input message), `NAME.out` (expected stdout with `NO_COLOR=1`), `NAME.code` (expected exit code as text, e.g. `1`).
  - assemble: `NAME.in` (key=value lines: `type=`, `scope=`, `desc=`, `body=` (literal `\n` for newlines), `breaking=` (0/1), `breaking_desc=`, `footer=`), `NAME.out` (expected assembled message).
  - config: `NAME.json` (a `.cmt.json`), `NAME.resolved` (canonical dump of resolved config — see Task 4 for the exact dump format).
  - hooks: `NAME.before` (pre-existing hook file or empty), `NAME.after` (expected file content after the operation), `NAME.op` (operation: `install-prepare`, `install-lint`, `remove-prepare`, `remove-lint`).

- [ ] **Step 1: Write `scripts/capture_fixtures.sh` to record lint fixtures**

```bash
#!/usr/bin/env bash
# One-time recorder: drives the current bash ./cmt to snapshot parity outputs.
# After this runs and fixtures are committed, the bash script is deleted.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BASH_CMT="$ROOT/cmt"
FX="$ROOT/tests/fixtures"
mkdir -p "$FX/lint"

emit_lint() { # name, message
  local name="$1" msg="$2"
  printf '%s' "$msg" > "$FX/lint/$name.msg"
  set +e
  NO_COLOR=1 bash "$BASH_CMT" lint "$FX/lint/$name.msg" > "$FX/lint/$name.out" 2>&1
  echo $? > "$FX/lint/$name.code"
  set -e
}

emit_lint valid_simple       'feat: add login'
emit_lint valid_scope        'fix(auth): handle null token'
emit_lint bad_format         'added a thing'
emit_lint unknown_type       'wibble: stuff'
emit_lint empty_desc         'feat: '
emit_lint upper_desc         'feat: Add login'
emit_lint trailing_period    'feat: add login.'
emit_lint long_header        'feat: this is an extremely long commit header that goes well beyond the seventy two character soft limit'
emit_lint body_no_blank      $'feat: x\nbody line'
emit_lint breaking_marker    'feat!: drop v1 api'
echo "lint fixtures captured"
```

- [ ] **Step 2: Run the recorder and inspect output**

Run: `chmod +x scripts/capture_fixtures.sh && ./scripts/capture_fixtures.sh && ls tests/fixtures/lint`
Expected: ten `.msg`/`.out`/`.code` triples exist. Spot-check `tests/fixtures/lint/bad_format.out` contains the header-format error line and `bad_format.code` is `1`.

- [ ] **Step 3: Extend the recorder for assemble fixtures**

Append to `scripts/capture_fixtures.sh` before the final echo:

```bash
mkdir -p "$FX/assemble"
# Assemble is internal; reproduce its exact rules here as the oracle.
# Order: header = type(scope)?!?: desc ; then blank+body ; blank+BREAKING ; blank+footer
emit_assemble() { # name type scope desc breaking breaking_desc footer body
  local name="$1" type="$2" scope="$3" desc="$4" brk="$5" bdesc="$6" footer="$7" body="$8"
  {
    echo "type=$type"; echo "scope=$scope"; echo "desc=$desc"
    echo "breaking=$brk"; echo "breaking_desc=$bdesc"; echo "footer=$footer"
    printf 'body=%s\n' "${body//$'\n'/\\n}"
  } > "$FX/assemble/$name.in"
  local header="$type"
  [ -n "$scope" ] && header="$type($scope)"
  [ "$brk" = "1" ] && header="$header!"
  header="$header: $desc"
  local msg="$header"
  [ -n "$body" ] && msg="$msg"$'\n\n'"$body"
  { [ "$brk" = "1" ] && [ -n "$bdesc" ]; } && msg="$msg"$'\n\n'"BREAKING CHANGE: $bdesc"
  [ -n "$footer" ] && msg="$msg"$'\n\n'"$footer"
  printf '%s' "$msg" > "$FX/assemble/$name.out"
}
emit_assemble simple   feat ""    "add login" 0 "" ""          ""
emit_assemble scoped   fix  auth  "null token" 0 "" ""         ""
emit_assemble body     feat ""    "add x"     0 "" ""          $'why this\nmatters'
emit_assemble breaking feat api   "new shape" 1 "drops old"    "Closes #9" "context"
emit_assemble footer   chore ""   "deps"      0 "" "Refs #1"   ""
```

- [ ] **Step 4: Run recorder again, verify assemble fixtures**

Run: `./scripts/capture_fixtures.sh && cat tests/fixtures/assemble/breaking.out`
Expected: header `feat(api)!: new shape`, blank line, `context`, blank line, `BREAKING CHANGE: drops old`, blank line, `Closes #9`.

- [ ] **Step 5: Extend the recorder for config fixtures (the `.resolved` dump matches Task 4's format exactly)**

Append:

```bash
mkdir -p "$FX/config"
# Each NAME.json is an input config; NAME.resolved is the canonical dump
# (format defined in Task 4). We hand-write .resolved here as the oracle.
cat > "$FX/config/empty.json" <<'JSON'
{}
JSON
cat > "$FX/config/empty.resolved" <<'TXT'
maxHeaderLength=72
requireScope=false
disallowUpperCaseDescription=false
disallowTrailingPeriod=false
allowBreakingChanges=feat,fix
scopes=
types=feat,fix,docs,style,refactor,perf,test,build,ci,chore,revert
TXT
cat > "$FX/config/custom.json" <<'JSON'
{
  "customTypes": [
    { "type": "wip", "emoji": "🚧", "semver": "none", "description": "Work in progress" }
  ],
  "scopes": ["auth", "api"],
  "rules": {
    "maxHeaderLength": 50,
    "requireScope": true,
    "disallowTrailingPeriod": true,
    "allowBreakingChanges": ["feat"]
  }
}
JSON
cat > "$FX/config/custom.resolved" <<'TXT'
maxHeaderLength=50
requireScope=true
disallowUpperCaseDescription=false
disallowTrailingPeriod=true
allowBreakingChanges=feat
scopes=auth,api
types=feat,fix,docs,style,refactor,perf,test,build,ci,chore,revert,wip
TXT
```

- [ ] **Step 6: Extend the recorder for hook fixtures (record exact snippet blocks)**

Append:

```bash
mkdir -p "$FX/hooks"
# Capture exact snippet text the bash script writes so the Rust port matches byte-for-byte.
# Source the bash script to access its private snippet functions.
CMT_SOURCED=1 source "$BASH_CMT"
{ printf '#!/usr/bin/env bash\n'; _hook_snippet; } > "$FX/hooks/prepare_fresh.after"
: > "$FX/hooks/prepare_fresh.before"
echo "install-prepare" > "$FX/hooks/prepare_fresh.op"
{ printf '#!/usr/bin/env bash\n'; _lint_snippet; } > "$FX/hooks/commitmsg_fresh.after"
: > "$FX/hooks/commitmsg_fresh.before"
echo "install-lint" > "$FX/hooks/commitmsg_fresh.op"
```

- [ ] **Step 7: Run recorder, verify hook snippet captured**

Run: `./scripts/capture_fixtures.sh && cat tests/fixtures/hooks/prepare_fresh.after`
Expected: file begins with `#!/usr/bin/env bash`, contains `# >>> cmt`, the binary probe loop, `cmt commit --write-only > "$1"`, and `# <<< cmt`.

- [ ] **Step 8: Delete the bash script and its test suite**

```bash
git rm cmt test/helpers.sh test/run_tests.sh test/test_assemble.sh test/test_config.sh test/test_lint.sh
rmdir test 2>/dev/null || true
```

- [ ] **Step 9: Verify the bash script is gone and fixtures remain**

Run: `test ! -f cmt && ls tests/fixtures/*/ | head`
Expected: `cmt` no longer exists; fixture directories populated.

- [ ] **Step 10: Commit**

```bash
git add tests/fixtures scripts/capture_fixtures.sh
git commit -m "test: capture parity fixtures and remove bash implementation"
```

---

## Task 3: Core types and builtin types

**Files:**
- Create: `src/types.rs`
- Modify: `src/main.rs` (add `mod types;`)
- Test: inline `#[cfg(test)]` in `src/types.rs`

**Interfaces:**
- Consumes: nothing.
- Produces:
  - `pub struct CommitType { pub r#type: String, pub emoji: String, pub semver: String, pub description: String }`
  - `pub struct Rules { pub max_header_length: usize, pub require_scope: bool, pub disallow_upper_case_description: bool, pub disallow_trailing_period: bool, pub allow_breaking_changes: Vec<String> }`
  - `pub struct Config { pub types: Vec<CommitType>, pub scopes: Vec<String>, pub rules: Rules }`
  - `pub fn builtin_types() -> Vec<CommitType>`
  - `impl Default for Rules` and `impl Default for Config`.

- [ ] **Step 1: Write the failing test**

In `src/types.rs`:

```rust
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
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib types`
Expected: FAIL — `builtin_types`/`Rules` not found.

- [ ] **Step 3: Implement `src/types.rs`**

```rust
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
        Config { types: builtin_types(), scopes: Vec::new(), rules: Rules::default() }
    }
}

pub fn builtin_types() -> Vec<CommitType> {
    // (type, emoji, semver, description) — order and emoji bytes are load-bearing.
    const RAW: &[(&str, &str, &str, &str)] = &[
        ("feat", "✨", "minor", "A new feature"),
        ("fix", "🐛", "patch", "A bug fix"),
        ("docs", "📚", "none", "Documentation only changes"),
        ("style", "💅", "none", "Formatting, missing semi-colons, etc — no logic change"),
        ("refactor", "♻️ ", "none", "Code change that neither fixes a bug nor adds a feature"),
        ("perf", "⚡", "patch", "A code change that improves performance"),
        ("test", "🧪", "none", "Adding or correcting tests"),
        ("build", "🏗️ ", "none", "Changes to build system or external dependencies"),
        ("ci", "🔧", "none", "Changes to CI/CD configuration files and scripts"),
        ("chore", "🔩", "none", "Other changes that don't modify src or test files"),
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
```

- [ ] **Step 4: Add `mod types;` to `src/main.rs`**

```rust
mod types;

fn main() {
    println!("cmt {}", env!("CARGO_PKG_VERSION"));
}
```

- [ ] **Step 5: Run tests to verify pass**

Run: `cargo test --lib types`
Expected: PASS (2 tests).

- [ ] **Step 6: Commit**

```bash
git add src/types.rs src/main.rs
git commit -m "feat: add core config types and builtin commit types"
```

---

## Task 4: Config loader (serde_json) + canonical dump for parity

**Files:**
- Create: `src/config.rs`
- Modify: `src/main.rs` (add `mod config;`)
- Test: `tests/parity_config.rs`

**Interfaces:**
- Consumes: `types::{Config, CommitType, Rules, builtin_types}` from Task 3.
- Produces:
  - `pub fn config_path() -> String` — returns `$CMT_CONFIG_FILE` or `.cmt.json`.
  - `pub fn load() -> Config` — loads `config_path()`; returns `Config::default()` if absent/invalid.
  - `pub fn load_from(path: &std::path::Path) -> Config`.
  - `pub fn dump_canonical(c: &Config) -> String` — the exact `.resolved` format used by fixtures (see below).

The canonical dump format (one key per line, newline-terminated, in this order):

```
maxHeaderLength=<n>
requireScope=<true|false>
disallowUpperCaseDescription=<true|false>
disallowTrailingPeriod=<true|false>
allowBreakingChanges=<comma-joined>
scopes=<comma-joined>
types=<comma-joined type names>
```

- [ ] **Step 1: Write the failing parity test**

`tests/parity_config.rs`:

```rust
use std::fs;
use std::path::Path;

#[test]
fn config_fixtures_resolve_canonically() {
    let dir = Path::new("tests/fixtures/config");
    let mut entries: Vec<_> = fs::read_dir(dir).unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "json").unwrap_or(false))
        .collect();
    entries.sort();
    assert!(!entries.is_empty(), "no config fixtures found");
    for json in entries {
        let resolved = json.with_extension("resolved");
        let expected = fs::read_to_string(&resolved).unwrap();
        let cfg = cmt::config::load_from(&json);
        let got = cmt::config::dump_canonical(&cfg);
        assert_eq!(got, expected, "mismatch for {:?}", json);
    }
}
```

This requires a library target. Add to `Cargo.toml` under `[lib]`:

```toml
[lib]
name = "cmt"
path = "src/lib.rs"
```

And create `src/lib.rs`:

```rust
pub mod types;
pub mod config;
```

(Later tasks add their modules to `src/lib.rs`; `src/main.rs` will `use cmt::...`.)

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test parity_config`
Expected: FAIL — `cmt::config` unresolved.

- [ ] **Step 3: Implement `src/config.rs`**

```rust
use std::path::Path;
use serde::Deserialize;
use crate::types::{Config, CommitType, Rules, builtin_types};

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
    let mut allow = raw.rules.allow_breaking_changes.unwrap_or_else(|| d.allow_breaking_changes.clone());
    if allow.is_empty() {
        allow = d.allow_breaking_changes.clone();
    }
    let rules = Rules {
        max_header_length: raw.rules.max_header_length.unwrap_or(d.max_header_length),
        require_scope: raw.rules.require_scope.unwrap_or(d.require_scope),
        disallow_upper_case_description: raw.rules.disallow_upper_case_description.unwrap_or(d.disallow_upper_case_description),
        disallow_trailing_period: raw.rules.disallow_trailing_period.unwrap_or(d.disallow_trailing_period),
        allow_breaking_changes: allow,
    };

    Config { types, scopes: raw.scopes, rules }
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
```

- [ ] **Step 4: Run test to verify pass**

Run: `cargo test --test parity_config`
Expected: PASS — both `empty` and `custom` fixtures resolve identically.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml src/lib.rs src/config.rs tests/parity_config.rs src/main.rs
git commit -m "feat: load .cmt.json config via serde with default merging"
```

---

## Task 5: UI primitives — palette, wordwrap, commit box

**Files:**
- Create: `src/ui.rs`
- Modify: `src/lib.rs` (add `pub mod ui;`)
- Test: inline tests in `src/ui.rs`

**Interfaces:**
- Consumes: nothing.
- Produces:
  - `pub struct Palette { pub reset, bold, dim, accent, accent_bold, green, yellow, red, blue, muted: &'static str }`
  - `pub fn palette() -> Palette` — all-empty strings when `NO_COLOR` set or stdout not a TTY; otherwise the truecolor codes.
  - `pub fn wordwrap(text: &str, width: usize) -> Vec<String>`
  - `pub fn commit_box(msg: &str, palette: &Palette, term_cols: usize) -> String`

- [ ] **Step 1: Write the failing test**

In `src/ui.rs`:

```rust
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
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib ui`
Expected: FAIL — `wordwrap`/`Palette` not found.

- [ ] **Step 3: Implement `src/ui.rs`**

```rust
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
            reset: "", bold: "", dim: "", accent: "", accent_bold: "",
            green: "", yellow: "", red: "", blue: "", muted: "",
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
    if std::io::stdout().is_terminal() {
        Palette::colored()
    } else {
        Palette::plain()
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
    out.push_str(&format!("\n  {}╭─ commit message {}╮{}\n", p.muted, bar, p.reset));
    for line in msg.lines() {
        for wrapped in wordwrap(line, content_w) {
            let pad = content_w.saturating_sub(wrapped.chars().count());
            out.push_str(&format!(
                "  {}│{}  {}{}{}{}  {}│{}\n",
                p.muted, p.reset, p.bold, wrapped, " ".repeat(pad), p.reset, p.muted, p.reset
            ));
        }
    }
    let bottom = "─".repeat(w.saturating_sub(2));
    out.push_str(&format!("  {}╰{}╯{}\n", p.muted, bottom, p.reset));
    out
}
```

- [ ] **Step 4: Run tests to verify pass**

Run: `cargo test --lib ui`
Expected: PASS (3 tests).

- [ ] **Step 5: Note on `unicode-width`**

If `commit_box` alignment looks wrong with wide emoji during manual testing (Task 10), add `unicode-width = "0.1"` to `Cargo.toml` and replace `.chars().count()` with display width in `commit_box`/`wordwrap`. Do not add the dep otherwise.

- [ ] **Step 6: Commit**

```bash
git add src/ui.rs src/lib.rs
git commit -m "feat: add color palette, wordwrap, and commit box rendering"
```

---

## Task 6: Lint engine (8 rules)

**Files:**
- Create: `src/lint.rs`
- Modify: `src/lib.rs` (add `pub mod lint;`)
- Test: `tests/parity_lint.rs`

**Interfaces:**
- Consumes: `config::load_from`, `types::Config`.
- Produces:
  - `pub struct LintResult { pub header: String, pub errors: Vec<String>, pub warnings: Vec<String> }`
  - `impl LintResult { pub fn exit_code(&self) -> i32 }` (1 if errors, else 0)
  - `pub fn check(msg: &str, cfg: &Config) -> LintResult`
  - `pub fn render(result: &LintResult, p: &ui::Palette) -> String` — the exact stdout the `lint` command prints.

Exact messages (must match fixtures captured in Task 2):
- header format: `Header must match: type(scope)?: description`
- unknown type: `Unknown type '<t>'. Valid: <comma-joined type names>`
- empty desc: `Description must not be empty`
- upper (error): `Description must start with a lowercase letter`; (warn): `Description starts with uppercase — prefer lowercase`
- period (error): `Description must not end with a period`; (warn): `Description ends with a period — omit it`
- header length (warn): `Header is <n> chars — aim for ≤<max>`
- require scope (error): `Scope is required — e.g. feat(auth): ...`
- blank line (error): `Blank line required between header and body`

- [ ] **Step 1: Write the failing parity test**

`tests/parity_lint.rs`:

```rust
use std::fs;
use std::path::Path;

#[test]
fn lint_fixtures_match() {
    // Fixtures were captured with NO_COLOR=1.
    std::env::set_var("NO_COLOR", "1");
    let dir = Path::new("tests/fixtures/lint");
    let mut names: Vec<String> = fs::read_dir(dir).unwrap()
        .filter_map(|e| e.ok())
        .filter_map(|e| e.path().file_stem().map(|s| s.to_string_lossy().into_owned()))
        .collect();
    names.sort();
    names.dedup();
    assert!(!names.is_empty());
    let cfg = cmt::types::Config::default();
    let p = cmt::ui::Palette::plain();
    for name in names {
        let msg = fs::read_to_string(dir.join(format!("{name}.msg"))).unwrap();
        let expected_out = fs::read_to_string(dir.join(format!("{name}.out"))).unwrap();
        let expected_code: i32 = fs::read_to_string(dir.join(format!("{name}.code")))
            .unwrap().trim().parse().unwrap();
        let result = cmt::lint::check(&msg, &cfg);
        let rendered = cmt::lint::render(&result, &p);
        assert_eq!(rendered, expected_out, "stdout mismatch for {name}");
        assert_eq!(result.exit_code(), expected_code, "exit code mismatch for {name}");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test parity_lint`
Expected: FAIL — `cmt::lint` unresolved.

- [ ] **Step 3: Implement `src/lint.rs` (check logic)**

```rust
use crate::types::Config;
use crate::ui::Palette;

pub struct LintResult {
    pub header: String,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl LintResult {
    pub fn exit_code(&self) -> i32 {
        if self.errors.is_empty() { 0 } else { 1 }
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
            if desc.chars().next().map(|c| c.is_ascii_uppercase()).unwrap_or(false) {
                if cfg.rules.disallow_upper_case_description {
                    errors.push("Description must start with a lowercase letter".to_string());
                } else {
                    warnings.push("Description starts with uppercase — prefer lowercase".to_string());
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
                warnings.push(format!("Header is {} chars — aim for ≤{}", hlen, cfg.rules.max_header_length));
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

    LintResult { header, errors, warnings }
}
```

- [ ] **Step 4: Implement `render` to match captured stdout**

Append to `src/lint.rs`. The bash output is (with `NO_COLOR=1`, codes are empty):

```
<newline>
  lint  <header>
  ✖  <error>            (per error)
  ⚑  <warning>          (per warning)
  ✔ valid conventional commit            (only if no errors AND no warnings)
<newline>  ✔ valid — with N suggestion(s)   (only if no errors AND >=1 warning)
<newline>
```

```rust
pub fn render(r: &LintResult, p: &Palette) -> String {
    let mut out = String::new();
    out.push_str(&format!("\n  {}lint{}  {}{}{}\n", p.accent_bold, p.reset, p.muted, r.header, p.reset));
    for e in &r.errors {
        out.push_str(&format!("  {}✖{}  {}\n", p.red, p.reset, e));
    }
    for w in &r.warnings {
        out.push_str(&format!("  {}⚑{}  {}{}{}\n", p.yellow, p.reset, p.yellow, w, p.reset));
    }
    if r.errors.is_empty() && r.warnings.is_empty() {
        out.push_str(&format!("  {}✔{} valid conventional commit\n", p.green, p.reset));
    } else if r.errors.is_empty() {
        out.push_str(&format!("\n  {}✔{} valid — with {} suggestion(s)\n", p.green, p.reset, r.warnings.len()));
    }
    out.push('\n');
    out
}
```

- [ ] **Step 5: Run tests to verify pass**

Run: `cargo test --test parity_lint`
Expected: PASS. If any fixture mismatches, diff `rendered` vs `<name>.out` and align spacing/wording exactly — fixtures are the source of truth.

- [ ] **Step 6: Commit**

```bash
git add src/lint.rs src/lib.rs tests/parity_lint.rs
git commit -m "feat: implement conventional commit lint engine"
```

---

## Task 7: Git wrappers

**Files:**
- Create: `src/git.rs`
- Modify: `src/lib.rs` (add `pub mod git;`)
- Test: inline test (smoke only — guarded to run inside this repo).

**Interfaces:**
- Consumes: nothing.
- Produces:
  - `pub fn in_repo() -> bool`
  - `pub fn top_level() -> Option<String>`
  - `pub fn has_staged_changes() -> bool`
  - `pub fn commit(message: &str, amend: bool) -> std::io::Result<std::process::ExitStatus>`
  - `pub fn log_lines(limit: usize) -> Vec<String>` — `git log -<n> --format="%H|%h|%ad|%an|%s" --date=short`
  - `pub fn last_message() -> Option<String>` — `git log -1 --format=%B`
  - `pub fn status_porcelain() -> Vec<String>`

- [ ] **Step 1: Write the failing test**

In `src/git.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn detects_repo_when_run_inside_one() {
        // This crate lives in a git repo, so this should be true in CI/dev.
        assert!(in_repo());
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib git`
Expected: FAIL — module/functions not found.

- [ ] **Step 3: Implement `src/git.rs`**

```rust
use std::process::{Command, ExitStatus, Stdio};

fn run(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).into_owned())
}

pub fn in_repo() -> bool {
    Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn top_level() -> Option<String> {
    run(&["rev-parse", "--show-toplevel"]).map(|s| s.trim_end().to_string())
}

pub fn has_staged_changes() -> bool {
    // `git diff --cached --quiet` exits 1 when there ARE staged changes.
    Command::new("git")
        .args(["diff", "--cached", "--quiet"])
        .status()
        .map(|s| !s.success())
        .unwrap_or(false)
}

pub fn commit(message: &str, amend: bool) -> std::io::Result<ExitStatus> {
    let mut cmd = Command::new("git");
    cmd.arg("commit");
    if amend {
        cmd.args(["--amend", "--no-edit"]);
    }
    cmd.args(["-m", message]).status()
}

pub fn log_lines(limit: usize) -> Vec<String> {
    run(&[
        "log",
        &format!("-{limit}"),
        "--format=%H|%h|%ad|%an|%s",
        "--date=short",
    ])
    .map(|s| s.lines().map(|l| l.to_string()).collect())
    .unwrap_or_default()
}

pub fn last_message() -> Option<String> {
    run(&["log", "-1", "--format=%B"])
}

pub fn status_porcelain() -> Vec<String> {
    run(&["status", "--porcelain"])
        .map(|s| s.lines().map(|l| l.to_string()).collect())
        .unwrap_or_default()
}
```

- [ ] **Step 4: Run test to verify pass**

Run: `cargo test --lib git`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/git.rs src/lib.rs
git commit -m "feat: add git subprocess wrappers"
```

---

## Task 8: Picker state machine (pure, no TTY)

**Files:**
- Create: `src/picker/state.rs`
- Create: `src/picker/mod.rs` (stub declaring `pub mod state;` for now)
- Modify: `src/lib.rs` (add `pub mod picker;`)
- Test: inline tests in `src/picker/state.rs`

**Interfaces:**
- Consumes: nothing.
- Produces:
  - `pub struct PickerState { pub items: Vec<String>, pub plain: Vec<String>, pub filter: String, pub cur: usize, pub top: usize, pub view: usize, pub visible: Vec<usize> }`
  - `pub fn new(items: Vec<String>) -> PickerState` (view defaults to 7; strips ANSI for `plain`)
  - `pub fn move_up(&mut self)`, `pub fn move_down(&mut self)`
  - `pub fn push_filter(&mut self, c: char)`, `pub fn backspace(&mut self)`, `pub fn clear_filter(&mut self)`
  - `pub fn selected_original(&self) -> Option<usize>` (maps `cur` -> original index via `visible`)
  - internal `fn rebuild_visible(&mut self)` (case-sensitive substring match on `plain`, matching bash)

Behavior parity with bash `_pick`:
- view window = 7; wrap-around on up/down; `top` adjusts so `cur` stays visible.
- filter matches against ANSI-stripped text, case-sensitive substring.
- after rebuild: clamp `cur` to last visible if it overflowed; `cur=0` and `top=0` when no matches.

- [ ] **Step 1: Write the failing tests**

In `src/picker/state.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn st() -> PickerState {
        new(vec!["feat".into(), "fix".into(), "docs".into(), "style".into(),
                 "refactor".into(), "perf".into(), "test".into(), "build".into()])
    }

    #[test]
    fn move_down_wraps() {
        let mut s = st();
        for _ in 0..s.items.len() { s.move_down(); }
        assert_eq!(s.cur, 0);
    }

    #[test]
    fn move_up_from_top_wraps_to_last() {
        let mut s = st();
        s.move_up();
        assert_eq!(s.cur, s.items.len() - 1);
    }

    #[test]
    fn filter_narrows_visible_and_clamps_cur() {
        let mut s = st();
        s.cur = 7;
        s.push_filter('f'); // matches feat, fix, refactor (contains f)
        assert_eq!(s.visible.iter().map(|&i| s.items[i].clone()).collect::<Vec<_>>(),
                   vec!["feat", "fix", "refactor"]);
        assert!(s.cur < s.visible.len());
    }

    #[test]
    fn selected_maps_to_original_index() {
        let mut s = st();
        s.push_filter('d'); // "docs", "build" contain d
        s.cur = 1;          // second visible = "build" at original index 7
        assert_eq!(s.selected_original(), Some(7));
    }

    #[test]
    fn no_match_yields_none() {
        let mut s = st();
        s.push_filter('z');
        assert!(s.visible.is_empty());
        assert_eq!(s.selected_original(), None);
    }

    #[test]
    fn strips_ansi_for_matching() {
        let s = new(vec!["\x1b[1mfeat\x1b[0m  badge".into()]);
        assert_eq!(s.plain[0], "feat  badge");
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --lib picker::state`
Expected: FAIL — module not found.

- [ ] **Step 3: Implement `src/picker/state.rs`**

```rust
pub struct PickerState {
    pub items: Vec<String>,
    pub plain: Vec<String>,
    pub filter: String,
    pub cur: usize,
    pub top: usize,
    pub view: usize,
    pub visible: Vec<usize>,
}

fn strip_ansi(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = String::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == 0x1b && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
            i += 2;
            while i < bytes.len() {
                let c = bytes[i] as char;
                i += 1;
                if c.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            // copy one UTF-8 char
            let ch_len = utf8_len(bytes[i]);
            out.push_str(std::str::from_utf8(&bytes[i..i + ch_len]).unwrap_or(""));
            i += ch_len;
        }
    }
    out
}

fn utf8_len(b: u8) -> usize {
    if b < 0x80 { 1 } else if b >> 5 == 0b110 { 2 } else if b >> 4 == 0b1110 { 3 } else { 4 }
}

pub fn new(items: Vec<String>) -> PickerState {
    let plain = items.iter().map(|s| strip_ansi(s)).collect();
    let mut s = PickerState {
        items,
        plain,
        filter: String::new(),
        cur: 0,
        top: 0,
        view: 7,
        visible: Vec::new(),
    };
    s.rebuild_visible();
    s
}

impl PickerState {
    fn rebuild_visible(&mut self) {
        self.visible.clear();
        for i in 0..self.items.len() {
            if self.filter.is_empty() || self.plain[i].contains(&self.filter) {
                self.visible.push(i);
            }
        }
        let nv = self.visible.len();
        if self.cur >= nv && nv > 0 {
            self.cur = nv - 1;
        }
        if nv == 0 {
            self.cur = 0;
        }
        self.top = 0;
    }

    pub fn move_up(&mut self) {
        let nv = self.visible.len();
        if nv == 0 { return; }
        self.cur = (self.cur + nv - 1) % nv;
        if self.cur == nv - 1 {
            self.top = nv.saturating_sub(self.view);
        } else if self.cur < self.top {
            self.top = self.cur;
        }
    }

    pub fn move_down(&mut self) {
        let nv = self.visible.len();
        if nv == 0 { return; }
        self.cur = (self.cur + 1) % nv;
        if self.cur >= self.top + self.view {
            self.top = self.cur + 1 - self.view;
        } else if self.cur == 0 {
            self.top = 0;
        }
    }

    pub fn push_filter(&mut self, c: char) {
        self.filter.push(c);
        self.rebuild_visible();
    }

    pub fn backspace(&mut self) {
        self.filter.pop();
        self.rebuild_visible();
    }

    pub fn clear_filter(&mut self) {
        self.filter.clear();
        self.rebuild_visible();
    }

    pub fn selected_original(&self) -> Option<usize> {
        self.visible.get(self.cur).copied()
    }
}
```

- [ ] **Step 4: Add `src/picker/mod.rs` stub**

```rust
pub mod state;
// render + key loop added in Task 9.
```

- [ ] **Step 5: Run tests to verify pass**

Run: `cargo test --lib picker::state`
Expected: PASS (6 tests).

- [ ] **Step 6: Commit**

```bash
git add src/picker/state.rs src/picker/mod.rs src/lib.rs
git commit -m "feat: implement pure picker state machine"
```

---

## Task 9: Picker render + key loop (crossterm) + fallback

This is the cross-platform arrow-nav core. No unit tests (it drives a real terminal); correctness of navigation logic is already covered by Task 8. Verify manually on each OS in Task 10 and via CI build on Windows.

**Files:**
- Modify: `src/picker/mod.rs`
- Test: manual (documented below).

**Interfaces:**
- Consumes: `picker::state::{PickerState, new}`, `ui::{Palette, palette}`.
- Produces:
  - `pub fn select(prompt: &str, items: &[String]) -> Option<usize>` — returns chosen original index, or `None` if no items / no match. Aborts the process with exit code 1 on Ctrl-C (matching bash).

Key map (crossterm `KeyEvent`):
- `Up` / `Char('k')` → `move_up`
- `Down` / `Char('j')` → `move_down`
- `Char(c)` (printable, not j/k handled above only when not filtering? — match bash: `j`/`k` ALWAYS navigate, never enter the filter) → `push_filter(c)` for other chars
- `Backspace` → `backspace`
- `Esc` → `clear_filter`
- `Enter` → confirm (break)
- `Char('q')` → if filter empty: abort(1); else `push_filter('q')`
- `Ctrl+C` → abort(1)

Note: in bash, `j`/`k` navigate even though they are printable — they never go into the filter. Preserve that.

- [ ] **Step 1: Implement `select` in `src/picker/mod.rs`**

```rust
pub mod state;

use std::io::Write;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute, queue,
    terminal::{self, Clear, ClearType},
};
use crate::ui::{palette, Palette};
use state::PickerState;

fn fallback(prompt: &str, items: &[String]) -> Option<usize> {
    use std::io::{stdin, stdout};
    let p = palette();
    println!("\n{}{}{}", p.bold, prompt, p.reset);
    for (i, item) in items.iter().enumerate() {
        println!("  {:>2}) {}", i + 1, item);
    }
    print!("\nChoice (1-{}): ", items.len());
    let _ = stdout().flush();
    let mut line = String::new();
    if stdin().read_line(&mut line).is_err() {
        return Some(0);
    }
    match line.trim().parse::<usize>() {
        Ok(n) if n >= 1 && n <= items.len() => Some(n - 1),
        _ => Some(0),
    }
}

fn abort() -> ! {
    let _ = terminal::disable_raw_mode();
    let mut out = std::io::stderr();
    let _ = execute!(out, cursor::Show);
    std::process::exit(1);
}

pub fn select(prompt: &str, items: &[String]) -> Option<usize> {
    if items.is_empty() {
        return None;
    }
    if terminal::enable_raw_mode().is_err() {
        return fallback(prompt, items);
    }
    let p = palette();
    let mut s = state::new(items.to_vec());
    let mut out = std::io::stderr();
    let _ = execute!(out, cursor::Hide);
    // header
    let _ = write!(out, "\n  {}{}{}\r\n\n", p.accent_bold, prompt, p.reset);
    let mut lines_drawn = draw(&mut out, &s, &p);

    loop {
        match event::read() {
            Ok(Event::Key(KeyEvent { code, modifiers, kind, .. }))
                if kind == KeyEventKind::Press =>
            {
                if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
                    abort();
                }
                match code {
                    KeyCode::Up | KeyCode::Char('k') => s.move_up(),
                    KeyCode::Down | KeyCode::Char('j') => s.move_down(),
                    KeyCode::Backspace => s.backspace(),
                    KeyCode::Esc => s.clear_filter(),
                    KeyCode::Enter => break,
                    KeyCode::Char('q') => {
                        if s.filter.is_empty() {
                            abort();
                        } else {
                            s.push_filter('q');
                        }
                    }
                    KeyCode::Char(c) => s.push_filter(c),
                    _ => {}
                }
                // move cursor back up over previously drawn lines, redraw
                let _ = queue!(out, cursor::MoveUp(lines_drawn as u16));
                lines_drawn = draw(&mut out, &s, &p);
            }
            Ok(_) => {}
            Err(_) => break,
        }
    }

    let chosen = s.selected_original();
    // collapse to the single selected line
    let _ = queue!(out, cursor::MoveUp(lines_drawn as u16), Clear(ClearType::FromCursorDown));
    if let Some(orig) = chosen {
        let _ = write!(out, "  {}❯{} {}{}{}\r\n", p.accent_bold, p.reset, p.bold, items[orig], p.reset);
    }
    let _ = execute!(out, cursor::Show);
    let _ = terminal::disable_raw_mode();
    chosen
}

// Draw the search line, scroll hints, and the visible window. Returns line count.
fn draw<W: Write>(out: &mut W, s: &PickerState, p: &Palette) -> usize {
    let mut n = 0;
    // search line
    if s.filter.is_empty() {
        let _ = write!(out, "  {}/ type to filter{}", p.muted, p.reset);
    } else {
        let _ = write!(out, "  {}/ {}{}", p.accent, s.filter, p.reset);
    }
    let _ = write!(out, "\x1b[K\r\n");
    n += 1;

    let nv = s.visible.len();
    let end = (s.top + s.view).min(nv);

    if s.top > 0 {
        let _ = write!(out, "  {}↑ {} more{}\x1b[K\r\n", p.muted, s.top, p.reset);
        n += 1;
    }

    if nv == 0 {
        let _ = write!(out, "  {}no matches{}\x1b[K\r\n", p.muted, p.reset);
        n += 1;
    } else {
        for i in s.top..end {
            let orig = s.visible[i];
            if i == s.cur {
                let _ = write!(out, "  {}❯{} {}{}\x1b[K\r\n", p.accent_bold, p.reset, s.items[orig], p.reset);
            } else {
                let _ = write!(out, "  {}·{} {}{}{}\x1b[K\r\n", p.muted, p.reset, p.muted, s.items[orig], p.reset);
            }
            n += 1;
        }
    }

    let remaining = nv.saturating_sub(end);
    if remaining > 0 {
        let _ = write!(out, "  {}↓ {} more{}\x1b[K\r\n", p.muted, remaining, p.reset);
    } else {
        let _ = write!(out, "\x1b[K\r\n");
    }
    n += 1;
    let _ = out.flush();
    n
}
```

- [ ] **Step 2: Build to verify it compiles on the dev machine**

Run: `cargo build`
Expected: compiles with no warnings.

- [ ] **Step 3: Manual smoke (documented; run after Task 15 wires `commit`)**

Manual test matrix (record results in the PR description):
- macOS Terminal.app + iTerm2: arrows move selection, typing filters, Esc clears, Enter selects, Ctrl-C aborts.
- Linux (gnome-terminal / xterm): same.
- Windows Terminal + PowerShell + cmd.exe: arrows move selection (THE key parity check).

- [ ] **Step 4: Commit**

```bash
git add src/picker/mod.rs
git commit -m "feat: implement crossterm picker render and key loop"
```

---

## Task 10: Commit wizard + assemble_message

**Files:**
- Create: `src/commit.rs`
- Modify: `src/lib.rs` (add `pub mod commit;`)
- Test: `tests/parity_assemble.rs` (assemble) + inline prompt-helper tests.

**Interfaces:**
- Consumes: `picker::select`, `config::load`, `types::Config`, `ui`, `git`.
- Produces:
  - `pub struct Draft { pub type_: String, pub scope: String, pub desc: String, pub body: String, pub breaking: bool, pub breaking_desc: String, pub footer: String }`
  - `pub fn assemble(d: &Draft) -> String`
  - `pub fn run(write_only: bool, dry_run: bool)` — full interactive flow + commit/preview.

`assemble` rules (must match `tests/fixtures/assemble`): header = `type` then `(scope)` if scope non-empty, then `!` if breaking, then `: desc`. Append `\n\n<body>` if body non-empty; `\n\nBREAKING CHANGE: <bd>` if breaking and bd non-empty; `\n\n<footer>` if footer non-empty.

- [ ] **Step 1: Write the failing assemble parity test**

`tests/parity_assemble.rs`:

```rust
use std::fs;
use std::path::Path;
use cmt::commit::{assemble, Draft};

fn parse_in(path: &Path) -> Draft {
    let mut d = Draft {
        type_: String::new(), scope: String::new(), desc: String::new(),
        body: String::new(), breaking: false, breaking_desc: String::new(),
        footer: String::new(),
    };
    for line in fs::read_to_string(path).unwrap().lines() {
        let (k, v) = line.split_once('=').unwrap_or((line, ""));
        match k {
            "type" => d.type_ = v.into(),
            "scope" => d.scope = v.into(),
            "desc" => d.desc = v.into(),
            "breaking" => d.breaking = v == "1",
            "breaking_desc" => d.breaking_desc = v.into(),
            "footer" => d.footer = v.into(),
            "body" => d.body = v.replace("\\n", "\n"),
            _ => {}
        }
    }
    d
}

#[test]
fn assemble_fixtures_match() {
    let dir = Path::new("tests/fixtures/assemble");
    let mut ins: Vec<_> = fs::read_dir(dir).unwrap().filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "in").unwrap_or(false))
        .collect();
    ins.sort();
    assert!(!ins.is_empty());
    for input in ins {
        let d = parse_in(&input);
        let expected = fs::read_to_string(input.with_extension("out")).unwrap();
        assert_eq!(assemble(&d), expected, "mismatch for {:?}", input);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test parity_assemble`
Expected: FAIL — `cmt::commit` unresolved.

- [ ] **Step 3: Implement `assemble` + `Draft` in `src/commit.rs`**

```rust
use std::io::{self, Write};
use crate::{config, git, picker, ui};
use crate::types::Config;

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
```

- [ ] **Step 4: Run assemble test to verify pass**

Run: `cargo test --test parity_assemble`
Expected: PASS.

- [ ] **Step 5: Implement the interactive prompts + `run`**

Append to `src/commit.rs`. These render to stderr (so `--write-only` stdout carries only the message, matching the bash hook contract). Build labels for the type picker exactly as bash `select_type` does (emoji, padded type, semver badge, description).

```rust
fn prompt_line(p: &ui::Palette, label: &str, hint: &str) -> String {
    let mut e = io::stderr();
    let _ = write!(e, "\n  {}{}{}  {}{}{}  {}›{} ", p.accent_bold, label, p.reset, p.muted, hint, p.reset, p.accent, p.reset);
    let _ = e.flush();
    read_line()
}

fn read_line() -> String {
    let mut s = String::new();
    let _ = io::stdin().read_line(&mut s);
    s.trim_end_matches(['\n', '\r']).to_string()
}

fn type_label(t: &crate::types::CommitType, p: &ui::Palette) -> String {
    let badge = match t.semver.as_str() {
        "minor" => format!("  {}[{}{}minor{}{}]{}", p.muted, p.reset, p.green, p.reset, p.muted, p.reset),
        "patch" => format!("  {}[{}{}patch{}{}]{}", p.muted, p.reset, p.yellow, p.reset, p.muted, p.reset),
        _ => "         ".to_string(), // 9 spaces — width of "[minor]"
    };
    format!("{}  {}{:<10}{}{}  {}{}{}", t.emoji, p.bold, t.r#type, p.reset, badge, p.muted, t.description, p.reset)
}

fn select_type(cfg: &Config, p: &ui::Palette) -> String {
    let labels: Vec<String> = cfg.types.iter().map(|t| type_label(t, p)).collect();
    let idx = picker::select("Select commit type:", &labels).unwrap_or(0);
    cfg.types[idx].r#type.clone()
}

fn select_scope(cfg: &Config, p: &ui::Palette) -> String {
    if !cfg.scopes.is_empty() {
        let mut labels: Vec<String> = cfg.scopes.iter()
            .map(|s| format!("{}{}{}", p.bold, s, p.reset)).collect();
        labels.push(format!("{}─  custom{}  {}type your own{}", p.muted, p.reset, p.muted, p.reset));
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
    let _ = write!(e, "\n  {}description{}  {}imperative, present tense{}\n", p.accent_bold, p.reset, p.muted, p.reset);
    let _ = e.flush();
    loop {
        let _ = write!(e, "  {}›{} ", p.accent, p.reset);
        let _ = e.flush();
        let desc = read_line();
        if desc.is_empty() {
            let _ = write!(e, "  {}⚑{}  {}Description is required{}\n", p.yellow, p.reset, p.yellow, p.reset);
            let _ = e.flush();
        } else if desc.chars().count() > cfg.rules.max_header_length {
            let _ = write!(e, "  {}⚑{}  {}Description is {} chars — keep it under {} for readability{}\n",
                p.yellow, p.reset, p.yellow, desc.chars().count(), cfg.rules.max_header_length, p.reset);
            let _ = write!(e, "  {}use anyway?{} {}[y/N]{}  {}›{} ", p.yellow, p.reset, p.muted, p.reset, p.accent, p.reset);
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
    let _ = write!(e, "\n  {}body{}  {}optional — empty line to finish{}\n", p.accent_bold, p.reset, p.muted, p.reset);
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
    let _ = write!(e, "\n  {}breaking change?{}  {}[y/N]{}  {}›{} ", p.accent_bold, p.reset, p.muted, p.reset, p.accent, p.reset);
    let _ = e.flush();
    let ans = read_line();
    if ans.eq_ignore_ascii_case("y") {
        let _ = write!(e, "  {}breaking change{}  {}›{} ", p.red, p.reset, p.accent, p.reset);
        let _ = e.flush();
        (true, read_line())
    } else {
        (false, String::new())
    }
}

fn input_footer(p: &ui::Palette) -> String {
    let mut e = io::stderr();
    let _ = write!(e, "\n  {}footer{}  {}e.g. Closes #42 — leave blank to skip{}\n", p.accent_bold, p.reset, p.muted, p.reset);
    let _ = write!(e, "  {}›{} ", p.accent, p.reset);
    let _ = e.flush();
    read_line()
}

fn term_cols() -> usize {
    crossterm::terminal::size().map(|(c, _)| c as usize).unwrap_or(80)
}

pub fn build_draft(cfg: &Config, p: &ui::Palette) -> Draft {
    let type_ = select_type(cfg, p);
    let scope = select_scope(cfg, p);
    let desc = input_description(cfg, p);
    let body = input_body(p);
    let (breaking, breaking_desc) = input_breaking(p);
    let footer = input_footer(p);
    Draft { type_, scope, desc, body, breaking, breaking_desc, footer }
}

pub fn run(write_only: bool, dry_run: bool) {
    let p = ui::palette();
    if !write_only && !dry_run {
        if !git::in_repo() {
            eprintln!("  {}✖{}  Not inside a git repository", p.red, p.reset);
            std::process::exit(1);
        }
        if !git::has_staged_changes() {
            eprintln!("  {}⚑{}  {}No staged changes detected. Did you forget `git add`?{}", p.yellow, p.reset, p.yellow, p.reset);
            std::process::exit(0);
        }
    }

    let mut e = io::stderr();
    let _ = write!(e, "\n  {}cmt{}  {}conventional commits  v{}{}\n", p.accent_bold, p.reset, p.muted, env!("CARGO_PKG_VERSION"), p.reset);
    let _ = write!(e, "  {}─────────────────────────────────────{}\n", p.muted, p.reset);
    let _ = e.flush();

    let cfg = config::load();
    let draft = build_draft(&cfg, &p);
    let msg = assemble(&draft);

    if write_only {
        eprint!("{}", ui::commit_box(&msg, &p, term_cols()));
        eprintln!();
        print!("{}", msg); // stdout: the message git will use
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
    let _ = write!(e, "\n  {}commit?{}  {}[Y/n/e]{}  {}›{} ", p.accent_bold, p.reset, p.muted, p.reset, p.accent, p.reset);
    let _ = e.flush();
    let ans = read_line();
    match ans.as_str() {
        "" | "y" | "Y" | "yes" | "Yes" | "YES" => {
            let _ = git::commit(msg, false);
            eprintln!("\n  {}✔{} {}committed{}", p.green, p.reset, p.bold, p.reset);
        }
        "e" | "E" | "edit" | "Edit" | "EDIT" => {
            let edited = edit_in_editor(msg);
            let _ = git::commit(&edited, false);
            eprintln!("\n  {}✔{} {}committed{}", p.green, p.reset, p.bold, p.reset);
        }
        _ => {
            eprintln!("  {}⚑{}  {}Commit aborted.{}", p.yellow, p.reset, p.yellow, p.reset);
            std::process::exit(0);
        }
    }
}

pub fn edit_in_editor(initial: &str) -> String {
    use std::process::Command;
    let dir = std::env::temp_dir();
    let path = dir.join(format!("cmt-commit-{}.txt", std::process::id()));
    let _ = std::fs::write(&path, initial);
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| {
        if cfg!(windows) { "notepad".into() } else { "vi".into() }
    });
    let _ = Command::new(editor).arg(&path).status();
    let edited = std::fs::read_to_string(&path).unwrap_or_else(|_| initial.to_string());
    let _ = std::fs::remove_file(&path);
    edited.trim_end_matches(['\n', '\r']).to_string()
}
```

- [ ] **Step 6: Build and run a manual dry-run after Task 15 wiring**

Run (after Task 15): stage a file, then `cargo run -- commit --dry-run`
Expected: wizard runs, arrow nav works, assembled message prints; no commit made.

- [ ] **Step 7: Commit**

```bash
git add src/commit.rs src/lib.rs tests/parity_assemble.rs
git commit -m "feat: implement interactive commit wizard and message assembly"
```

---

## Task 11: Hook snippets + marker-block install/remove

**Files:**
- Create: `src/hooks.rs`
- Modify: `src/lib.rs` (add `pub mod hooks;`)
- Test: `tests/parity_hooks.rs` + inline tests.

**Interfaces:**
- Consumes: nothing (pure file/string ops).
- Produces:
  - `pub fn prepare_snippet() -> String` (exact text, no trailing `#!`)
  - `pub fn lint_snippet() -> String`
  - `pub fn install_block(path: &Path, snippet: &str, marker: &str, add_shebang: bool) -> io::Result<Outcome>`
  - `pub fn remove_block(path: &Path, marker: &str) -> io::Result<Outcome>`
  - `pub enum Outcome { Created, Appended, Updated, Removed, RemovedFile, NoOp }`

Snippet text must match `tests/fixtures/hooks/*.after` byte-for-byte (after the shebang line). The marker for the prepare hook is `cmt`; for lint, `cmt-lint`. `install_block`: if file exists and contains `# >>> <marker>`, replace the block (awk-style: drop lines from `# >>> marker` through `# <<< marker`, then append fresh snippet); if exists without marker, append; if absent, create (with shebang when `add_shebang`). `remove_block`: strip the block; if the remaining file has no non-comment, non-blank, non-shebang content, delete the file.

- [ ] **Step 1: Write the failing parity test**

`tests/parity_hooks.rs`:

```rust
use std::fs;
use std::path::Path;
use cmt::hooks;

#[test]
fn fresh_install_matches_fixture() {
    let tmp = std::env::temp_dir().join(format!("cmt-hooktest-{}", std::process::id()));
    fs::create_dir_all(&tmp).unwrap();
    let path = tmp.join("prepare-commit-msg");

    hooks::install_block(&path, &hooks::prepare_snippet(), "cmt", true).unwrap();
    let got = fs::read_to_string(&path).unwrap();
    let expected = fs::read_to_string(Path::new("tests/fixtures/hooks/prepare_fresh.after")).unwrap();
    assert_eq!(got, expected);

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn fresh_lint_install_matches_fixture() {
    let tmp = std::env::temp_dir().join(format!("cmt-hooktest2-{}", std::process::id()));
    fs::create_dir_all(&tmp).unwrap();
    let path = tmp.join("commit-msg");
    hooks::install_block(&path, &hooks::lint_snippet(), "cmt-lint", true).unwrap();
    let got = fs::read_to_string(&path).unwrap();
    let expected = fs::read_to_string(Path::new("tests/fixtures/hooks/commitmsg_fresh.after")).unwrap();
    assert_eq!(got, expected);
    let _ = fs::remove_dir_all(&tmp);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test parity_hooks`
Expected: FAIL — `cmt::hooks` unresolved.

- [ ] **Step 3: Implement `src/hooks.rs`**

The snippet strings below MUST equal what Task 2 captured. Verify against `tests/fixtures/hooks/prepare_fresh.after` and adjust whitespace to match if needed.

```rust
use std::io;
use std::path::Path;

#[derive(Debug, PartialEq, Eq)]
pub enum Outcome { Created, Appended, Updated, Removed, RemovedFile, NoOp }

pub fn prepare_snippet() -> String {
    // Keep byte-identical to fixtures/hooks/prepare_fresh.after (minus shebang).
    r#"# >>> cmt — Conventional Commits CLI
[ -n "${2:-}" ] && exit 0
_cmt_bin=""
for _p in "./node_modules/.bin/cmt" "$HOME/.local/bin/cmt" "/usr/local/bin/cmt" "/opt/homebrew/bin/cmt"; do
  [ -x "$_p" ] && { _cmt_bin="$_p"; break; }
done
[ -z "$_cmt_bin" ] && _cmt_bin=$(command -v cmt 2>/dev/null)
[ -n "$_cmt_bin" ] && { "$_cmt_bin" commit --write-only > "$1"; exit 0; }
exit 0
# <<< cmt
"#.to_string()
}

pub fn lint_snippet() -> String {
    r#"# >>> cmt-lint — Conventional Commits CLI
_cmt_bin=""
for _p in "./node_modules/.bin/cmt" "$HOME/.local/bin/cmt" "/usr/local/bin/cmt" "/opt/homebrew/bin/cmt"; do
  [ -x "$_p" ] && { _cmt_bin="$_p"; break; }
done
[ -z "$_cmt_bin" ] && _cmt_bin=$(command -v cmt 2>/dev/null)
[ -n "$_cmt_bin" ] && { "$_cmt_bin" lint "$1"; exit $?; }
exit 0
# <<< cmt-lint
"#.to_string()
}

fn make_executable(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = std::fs::metadata(path) {
            let mut perm = meta.permissions();
            perm.set_mode(perm.mode() | 0o755);
            let _ = std::fs::set_permissions(path, perm);
        }
    }
    #[cfg(not(unix))]
    { let _ = path; }
}

// Remove lines from "# >>> marker" through "# <<< marker" (inclusive).
fn strip_block(content: &str, marker: &str) -> String {
    let open = format!("# >>> {marker}");
    let close = format!("# <<< {marker}");
    let mut out = String::new();
    let mut skip = false;
    for line in content.lines() {
        if line.starts_with(&open) {
            skip = true;
            continue;
        }
        if skip {
            if line == close {
                skip = false;
            }
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

pub fn install_block(path: &Path, snippet: &str, marker: &str, add_shebang: bool) -> io::Result<Outcome> {
    let open = format!("# >>> {marker}");
    let outcome;
    if path.exists() {
        let existing = std::fs::read_to_string(path)?;
        if existing.contains(&open) {
            let mut base = strip_block(&existing, marker);
            base.push_str(snippet);
            std::fs::write(path, base)?;
            outcome = Outcome::Updated;
        } else {
            let mut combined = existing;
            combined.push_str(snippet);
            std::fs::write(path, combined)?;
            outcome = Outcome::Appended;
        }
    } else {
        let mut content = String::new();
        if add_shebang {
            content.push_str("#!/usr/bin/env bash\n");
        }
        content.push_str(snippet);
        std::fs::write(path, content)?;
        outcome = Outcome::Created;
    }
    make_executable(path);
    Ok(outcome)
}

pub fn remove_block(path: &Path, marker: &str) -> io::Result<Outcome> {
    if !path.exists() {
        return Ok(Outcome::NoOp);
    }
    let existing = std::fs::read_to_string(path)?;
    let open = format!("# >>> {marker} ");
    if !existing.contains(&open) {
        return Ok(Outcome::NoOp);
    }
    let stripped = strip_block(&existing, marker);
    let has_content = stripped.lines().any(|l| {
        let t = l.trim();
        !t.is_empty() && !t.starts_with('#')
    });
    if has_content {
        std::fs::write(path, stripped)?;
        Ok(Outcome::Removed)
    } else {
        std::fs::remove_file(path)?;
        Ok(Outcome::RemovedFile)
    }
}
```

Note: `remove_block` uses `"# >>> {marker} "` (trailing space) so the `cmt` marker does not match `cmt-lint`, matching the bash `awk` guard.

- [ ] **Step 4: Run tests to verify pass**

Run: `cargo test --test parity_hooks`
Expected: PASS. If byte mismatch, align the snippet raw strings to the fixture exactly.

- [ ] **Step 5: Add inline tests for replace + remove idempotency**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn replace_is_idempotent() {
        let dir = std::env::temp_dir().join(format!("cmt-hk-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("prepare-commit-msg");
        install_block(&path, &prepare_snippet(), "cmt", true).unwrap();
        let once = std::fs::read_to_string(&path).unwrap();
        let out = install_block(&path, &prepare_snippet(), "cmt", true).unwrap();
        assert_eq!(out, Outcome::Updated);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), once);
        let r = remove_block(&path, "cmt").unwrap();
        assert_eq!(r, Outcome::RemovedFile);
        assert!(!path.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
```

- [ ] **Step 6: Run and commit**

Run: `cargo test --lib hooks && cargo test --test parity_hooks`
Expected: PASS.

```bash
git add src/hooks.rs src/lib.rs tests/parity_hooks.rs
git commit -m "feat: implement git hook snippet install and removal"
```

---

## Task 12: `init` and `uninstall` commands

**Files:**
- Create: `src/commands/mod.rs` (declares submodules)
- Create: `src/commands/init.rs`
- Create: `src/commands/uninstall.rs`
- Modify: `src/lib.rs` (add `pub mod commands;`)
- Test: inline integration tests using a temp git repo.

**Interfaces:**
- Consumes: `git::top_level`, `hooks::*`, `ui`.
- Produces:
  - `pub fn run(args: &[String])` in each of `init` and `uninstall`.
  - `init` writes `.cmt.json` (if absent) with the exact template, then installs hooks. `--husky` → `.husky/` (no shebang); else `.git/hooks/` (with shebang). `--lint` → also install the commit-msg/lint hook.
  - `uninstall` removes cmt blocks from `.git/hooks/{prepare-commit-msg,commit-msg}` and `.husky/{prepare-commit-msg,commit-msg}`, removes legacy `Installed by cmt` hooks, removes `.cmt.json`.

- [ ] **Step 1: Write the `.cmt.json` template constant + failing test**

`src/commands/init.rs`:

```rust
use std::path::Path;
use crate::{git, hooks, ui};

pub const SCHEMA_URL: &str = "https://raw.githubusercontent.com/mihai-ro/cmt/main/schema/cmt.schema.json";

pub fn config_template() -> String {
    format!(
r#"{{
  "$schema": "{SCHEMA_URL}",
  "customTypes": [
    {{ "type": "wip", "emoji": "🚧", "semver": "none", "description": "Work in progress" }}
  ],
  "scopes": [],
  "rules": {{
    "maxHeaderLength": 72,
    "requireScope": false,
    "allowBreakingChanges": ["feat", "fix"],
    "disallowUpperCaseDescription": false,
    "disallowTrailingPeriod": false
  }}
}}
"#)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn template_is_valid_json_and_has_schema() {
        let v: serde_json::Value = serde_json::from_str(&config_template()).unwrap();
        assert!(v.get("$schema").is_some());
        assert_eq!(v["rules"]["maxHeaderLength"], 72);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib commands::init`
Expected: FAIL — module not wired.

- [ ] **Step 3: Implement `src/commands/init.rs` `run`**

```rust
pub fn run(args: &[String]) {
    let p = ui::palette();
    if !git::in_repo() {
        eprintln!("  {}✖{}  Not inside a git repository", p.red, p.reset);
        std::process::exit(1);
    }
    let root = git::top_level().unwrap_or_else(|| ".".to_string());
    let root = Path::new(&root);

    let cfg_path = root.join(".cmt.json");
    if !cfg_path.exists() {
        let _ = std::fs::write(&cfg_path, config_template());
        eprintln!("  {}✔{} Created .cmt.json", p.green, p.reset);
    }

    let use_husky = args.iter().any(|a| a == "--husky");
    let use_lint = args.iter().any(|a| a == "--lint");

    if use_husky {
        let dir = root.join(".husky");
        let _ = std::fs::create_dir_all(&dir);
        let _ = hooks::install_block(&dir.join("prepare-commit-msg"), &hooks::prepare_snippet(), "cmt", false);
        eprintln!("  {}✔{} Created .husky/prepare-commit-msg", p.green, p.reset);
        if use_lint {
            let _ = hooks::install_block(&dir.join("commit-msg"), &hooks::lint_snippet(), "cmt-lint", false);
            eprintln!("  {}✔{} Created .husky/commit-msg", p.green, p.reset);
        }
        eprintln!("  {}Commit .husky/ hooks to share with your team.{}", p.dim, p.reset);
    } else {
        let dir = root.join(".git").join("hooks");
        let _ = std::fs::create_dir_all(&dir);
        let _ = hooks::install_block(&dir.join("prepare-commit-msg"), &hooks::prepare_snippet(), "cmt", true);
        eprintln!("  {}✔{} Created .git/hooks/prepare-commit-msg", p.green, p.reset);
        if use_lint {
            let _ = hooks::install_block(&dir.join("commit-msg"), &hooks::lint_snippet(), "cmt-lint", true);
            eprintln!("  {}✔{} Created .git/hooks/commit-msg", p.green, p.reset);
        }
    }

    eprintln!("\n  {}✔{} Done.", p.green, p.reset);
    eprintln!("  {}git commit  or  cmt commit — both work{}\n", p.dim, p.reset);
}
```

- [ ] **Step 4: Implement `src/commands/uninstall.rs`**

```rust
use std::path::Path;
use crate::{git, hooks, ui};

pub fn run(_args: &[String]) {
    let p = ui::palette();
    if !git::in_repo() {
        eprintln!("  {}✖{}  Not inside a git repository", p.red, p.reset);
        std::process::exit(1);
    }
    let root = git::top_level().unwrap_or_else(|| ".".to_string());
    let root = Path::new(&root);
    let mut removed = false;

    for dir in [root.join(".git").join("hooks"), root.join(".husky")] {
        if let Ok(o) = hooks::remove_block(&dir.join("prepare-commit-msg"), "cmt") {
            if o != hooks::Outcome::NoOp { removed = true; }
        }
        if let Ok(o) = hooks::remove_block(&dir.join("commit-msg"), "cmt-lint") {
            if o != hooks::Outcome::NoOp { removed = true; }
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
        eprintln!("  {}✔{} Removed .cmt.json", p.green, p.reset);
        removed = true;
    }

    if !removed {
        eprintln!("  {}·{} Nothing to remove.", p.blue, p.reset);
    }
    println!();
}
```

- [ ] **Step 5: Create `src/commands/mod.rs`**

```rust
pub mod init;
pub mod uninstall;
pub mod log;
pub mod types_cmd;
pub mod status;
pub mod amend;
pub mod completions;
pub mod help;
```

(Tasks 13–14 create the remaining submodules; if executing strictly in order, temporarily comment out not-yet-created lines and uncomment as you add them, or create empty stubs now.)

- [ ] **Step 6: Add round-trip integration test**

`tests/init_uninstall.rs`:

```rust
use std::process::Command;

#[test]
fn init_then_uninstall_round_trip() {
    let tmp = std::env::temp_dir().join(format!("cmt-it-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();
    Command::new("git").args(["init"]).current_dir(&tmp).status().unwrap();

    let bin = env!("CARGO_BIN_EXE_cmt");
    Command::new(bin).arg("init").current_dir(&tmp).status().unwrap();
    assert!(tmp.join(".cmt.json").exists());
    assert!(tmp.join(".git/hooks/prepare-commit-msg").exists());

    Command::new(bin).arg("uninstall").current_dir(&tmp).status().unwrap();
    assert!(!tmp.join(".cmt.json").exists());
    assert!(!tmp.join(".git/hooks/prepare-commit-msg").exists());

    let _ = std::fs::remove_dir_all(&tmp);
}
```

- [ ] **Step 7: Run tests to verify pass**

Run: `cargo test --test init_uninstall && cargo test --lib commands::init`
Expected: PASS. (Requires Task 15's dispatch so the binary handles `init`/`uninstall`; if running before Task 15, mark this step pending until dispatch exists.)

- [ ] **Step 8: Commit**

```bash
git add src/commands/ src/lib.rs tests/init_uninstall.rs
git commit -m "feat: implement init and uninstall commands"
```

---

## Task 13: `log`, `types`, `status`, `amend` commands

**Files:**
- Create: `src/commands/log.rs`, `src/commands/types_cmd.rs`, `src/commands/status.rs`, `src/commands/amend.rs`
- Test: inline tests for the pure pieces (log line formatting, status glyph mapping).

**Interfaces:**
- Consumes: `git`, `config`, `commit::{build_draft, assemble, edit_in_editor}`, `ui`.
- Produces: `pub fn run(args: &[String])` in each.

- [ ] **Step 1: Write failing tests for the pure formatters**

`src/commands/log.rs`:

```rust
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
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib commands::log`
Expected: FAIL.

- [ ] **Step 3: Implement `src/commands/log.rs`**

```rust
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
    // must be followed by '(' ':' or '!'
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
    if !filter_type.is_empty() { label.push_str(&format!("  {}type={}{}", p.muted, filter_type, p.reset)); }
    if !filter_author.is_empty() { label.push_str(&format!("  {}author={}{}", p.muted, filter_author, p.reset)); }
    println!("\n  {}log{}  {}{}{}", p.accent_bold, p.reset, p.muted, label, p.reset);

    for line in git::log_lines(limit) {
        let parts: Vec<&str> = line.splitn(5, '|').collect();
        if parts.len() < 5 { continue; }
        let (_hash, short, date, author, subject) = (parts[0], parts[1], parts[2], parts[3], parts[4]);
        let ty = parse_type(subject).unwrap_or_default();
        if !filter_type.is_empty() && ty != filter_type { continue; }
        if !filter_author.is_empty() && !author.contains(&filter_author) { continue; }

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
        let breaking = if is_breaking(subject) { format!(" {}breaking{}", p.red, p.reset) } else { String::new() };
        println!("  {}{}{}  {}  {}{}{}{}{}", p.muted, short, p.reset, emoji, color, p.bold, subject, p.reset, breaking);
        println!("  {}{}  {}{}\n", p.muted, date, author, p.reset);
    }
}
```

- [ ] **Step 4: Implement `src/commands/types_cmd.rs`**

```rust
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
        println!("  {}  {}{:<12}{}{}  {}{}{}", t.emoji, p.bold, t.r#type, p.reset, badge, p.muted, t.description, p.reset);
    }
    println!();
    if std::path::Path::new(&config::config_path()).exists() {
        println!("  {}custom types from .cmt.json included above{}\n", p.muted, p.reset);
    } else {
        println!("  {}run cmt init to configure custom types{}\n", p.muted, p.reset);
    }
}
```

- [ ] **Step 5: Implement `src/commands/status.rs`**

```rust
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
        if line.len() < 4 { continue; }
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
        println!("  {}{}{}  {:<40}  {}{}{}", color, glyph, p.reset, file, p.muted, word, p.reset);
        staged += 1;
    }
    if staged == 0 {
        eprintln!("  {}⚑{}  {}Nothing staged. Run: git add <file>{}", p.yellow, p.reset, p.yellow, p.reset);
    }
    println!();
}
```

- [ ] **Step 6: Implement `src/commands/amend.rs`**

```rust
use std::io::{self, Write};
use crate::{commit, config, git, ui};

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
    let cols = crossterm::terminal::size().map(|(c, _)| c as usize).unwrap_or(80);
    eprint!("{}", ui::commit_box(&last, &p, cols));

    let mut e = io::stderr();
    let _ = write!(e, "\n  {}amend{}  {}[Y=rebuild / e=edit raw / N=abort]{}  {}›{} ",
        p.accent_bold, p.reset, p.muted, p.reset, p.accent, p.reset);
    let _ = e.flush();
    let ans = read_line();

    match ans.as_str() {
        "e" | "E" => {
            let edited = commit::edit_in_editor(&last);
            let _ = git::commit(&edited, true);
            eprintln!("\n  {}✔{} {}amended{}", p.green, p.reset, p.bold, p.reset);
        }
        "" | "y" | "Y" => {
            let cfg = config::load();
            let draft = commit::build_draft(&cfg, &p);
            let msg = commit::assemble(&draft);
            eprint!("{}", ui::commit_box(&msg, &p, cols));
            let _ = write!(e, "\n  {}commit --amend?{}  {}[Y/n]{}  {}›{} ",
                p.accent_bold, p.reset, p.muted, p.reset, p.accent, p.reset);
            let _ = e.flush();
            let c = read_line();
            if c.eq_ignore_ascii_case("n") {
                eprintln!("  {}⚑{}  {}Amend aborted.{}", p.yellow, p.reset, p.yellow, p.reset);
                std::process::exit(0);
            }
            let _ = git::commit(&msg, true);
            eprintln!("\n  {}✔{} {}amended{}", p.green, p.reset, p.bold, p.reset);
        }
        _ => {
            eprintln!("  {}⚑{}  {}Amend aborted.{}", p.yellow, p.reset, p.yellow, p.reset);
            std::process::exit(0);
        }
    }
}
```

- [ ] **Step 7: Run tests to verify pass**

Run: `cargo test --lib commands::log`
Expected: PASS.

- [ ] **Step 8: Commit**

```bash
git add src/commands/log.rs src/commands/types_cmd.rs src/commands/status.rs src/commands/amend.rs
git commit -m "feat: implement log, types, status, and amend commands"
```

---

## Task 14: `completions`, `help`, `version`

**Files:**
- Create: `src/commands/completions.rs`
- Create: `src/commands/help.rs`
- Test: inline test asserting completions name all commands.

**Interfaces:**
- Consumes: `ui`.
- Produces:
  - `completions::run(args: &[String])` — prints bash/zsh/fish script (default bash); unknown shell → error + exit 1.
  - `help::print_help()`, `help::print_version()`.

- [ ] **Step 1: Write the failing test**

`src/commands/completions.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bash_lists_core_commands() {
        let s = bash();
        for c in ["commit", "lint", "init", "log", "types", "status", "amend", "uninstall", "completions", "version", "help"] {
            assert!(s.contains(c), "missing {c}");
        }
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib commands::completions`
Expected: FAIL.

- [ ] **Step 3: Implement `src/commands/completions.rs`**

```rust
pub fn bash() -> String {
    r#"# cmt bash completions — add to ~/.bashrc:  eval "$(cmt completions bash)"
_cmt_completions() {
  local cur="${COMP_WORDS[COMP_CWORD]}"
  local cmds="commit lint init log types status amend uninstall completions version help"
  COMPREPLY=( $(compgen -W "$cmds" -- "$cur") )
}
complete -F _cmt_completions cmt
"#.to_string()
}

pub fn zsh() -> String {
    r#"# cmt zsh completions — add to ~/.zshrc:  eval "$(cmt completions zsh)"
_cmt() {
  local -a cmds
  cmds=(
    'commit:Interactive commit builder'
    'lint:Lint a commit message file or stdin'
    'init:Set up repo with hook and config'
    'log:Pretty log of recent commits'
    'types:List available commit types'
    'status:Show staged changes'
    'amend:Amend the last commit'
    'uninstall:Remove cmt-managed hooks'
    'completions:Print shell completion scripts'
    'version:Print version'
    'help:Show help'
  )
  _describe 'command' cmds
}
compdef _cmt cmt
"#.to_string()
}

pub fn fish() -> String {
    r#"# cmt fish completions — add to config.fish:  cmt completions fish | source
complete -c cmt -f
complete -c cmt -n '__fish_use_subcommand' -a commit     -d 'Interactive commit builder'
complete -c cmt -n '__fish_use_subcommand' -a lint       -d 'Lint a commit message'
complete -c cmt -n '__fish_use_subcommand' -a init       -d 'Set up repo with hook and config'
complete -c cmt -n '__fish_use_subcommand' -a log        -d 'Pretty log of recent commits'
complete -c cmt -n '__fish_use_subcommand' -a types      -d 'List available commit types'
complete -c cmt -n '__fish_use_subcommand' -a status     -d 'Show staged changes'
complete -c cmt -n '__fish_use_subcommand' -a amend      -d 'Amend the last commit'
complete -c cmt -n '__fish_use_subcommand' -a uninstall  -d 'Remove cmt-managed hooks'
complete -c cmt -n '__fish_use_subcommand' -a completions -d 'Print shell completion scripts'
complete -c cmt -n '__fish_use_subcommand' -a version    -d 'Print version'
complete -c cmt -n '__fish_use_subcommand' -a help       -d 'Show help'
"#.to_string()
}

pub fn run(args: &[String]) {
    let shell = args.first().map(|s| s.as_str()).unwrap_or("bash");
    match shell {
        "bash" => print!("{}", bash()),
        "zsh" => print!("{}", zsh()),
        "fish" => print!("{}", fish()),
        other => {
            eprintln!("  ✖  Unknown shell: {other}  (use: bash, zsh, or fish)");
            std::process::exit(1);
        }
    }
}
```

- [ ] **Step 4: Implement `src/commands/help.rs`**

Port the bash help text verbatim (using the palette). Keep `cmt v<version>` from `CARGO_PKG_VERSION`.

```rust
use crate::ui;

pub fn print_version() {
    println!("cmt version {}", env!("CARGO_PKG_VERSION"));
}

pub fn print_help() {
    let p = ui::palette();
    let v = env!("CARGO_PKG_VERSION");
    print!(
"\n{bold}{cyan}cmt{reset} v{v} — Conventional Commits CLI\n\n{bold}USAGE{reset}\n  cmt <command> [options]\n\n{bold}COMMANDS{reset}\n  {cyan}init{reset} [--husky] [--lint]       Create .cmt.json + install prepare-commit-msg hook\n                        --husky  write to .husky/  (husky v9, committable)\n                        --lint   also install commit-msg linting hook\n                        default  write to .git/hooks/  (local)\n\n  {cyan}commit{reset} [--dry-run]         Interactive commit builder\n                        --dry-run  show assembled message, do not commit\n\n  {cyan}amend{reset}                    Amend the last commit (rebuild or raw edit)\n\n  {cyan}status{reset}                   Show staged changes\n\n  {cyan}lint{reset} [file]              Lint a commit message file or stdin\n\n  {cyan}log{reset} [n] [--type=<t>]     Pretty log — last n commits  (default: 20)\n      [--author=<a>]     filter by type or author substring\n\n  {cyan}types{reset}                    List available types\n\n  {cyan}completions{reset} [bash|zsh|fish]  Print shell completion script\n\n  {cyan}uninstall{reset}                Remove cmt-managed hooks\n\n{bold}EXAMPLES{reset}\n  cmt init                       # set up repo — done in one step\n  cmt commit                     # guided commit\n  cmt commit --dry-run           # preview message without committing\n  cmt amend                      # fix last commit\n  cmt log 10 --type=feat         # last 10 feat commits\n  echo 'fix: typo' | cmt lint\n\n{bold}SHELL COMPLETIONS{reset}\n  bash   Add to ~/.bashrc:    eval \"$(cmt completions bash)\"\n  zsh    Add to ~/.zshrc:     eval \"$(cmt completions zsh)\"\n  fish   Add to config.fish:  cmt completions fish | source\n\n{bold}CONFIG{reset}  .cmt.json  (JSON Schema -> intellisense in VS Code / JetBrains)\n\n",
        bold = p.bold, cyan = p.accent, reset = p.reset, v = v
    );
}
```

- [ ] **Step 5: Run tests to verify pass**

Run: `cargo test --lib commands::completions`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/commands/completions.rs src/commands/help.rs
git commit -m "feat: implement completions, help, and version output"
```

---

## Task 15: Main dispatch

**Files:**
- Modify: `src/main.rs`
- Modify: `src/lib.rs` (ensure all modules are `pub`)
- Test: `tests/cli_smoke.rs`

**Interfaces:**
- Consumes: every command module.
- Produces: full CLI. Default command (no args) = `commit`. Aliases per Global Constraints. `lint` reads a file arg or stdin.

- [ ] **Step 1: Write failing CLI smoke tests**

`tests/cli_smoke.rs`:

```rust
use std::process::Command;

fn bin() -> &'static str { env!("CARGO_BIN_EXE_cmt") }

#[test]
fn version_flag_prints_version() {
    let out = Command::new(bin()).arg("--version").output().unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("cmt version"));
}

#[test]
fn lint_stdin_exit_codes() {
    use std::io::Write;
    let mut child = Command::new(bin()).arg("lint")
        .env("NO_COLOR", "1")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn().unwrap();
    child.stdin.take().unwrap().write_all(b"feat: ok").unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());

    let mut child = Command::new(bin()).arg("lint")
        .env("NO_COLOR", "1")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn().unwrap();
    child.stdin.take().unwrap().write_all(b"nope").unwrap();
    let out = child.wait_with_output().unwrap();
    assert_eq!(out.status.code(), Some(1));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test cli_smoke`
Expected: FAIL — dispatch not implemented.

- [ ] **Step 3: Implement `src/main.rs`**

```rust
use std::io::Read;
use cmt::{commit, config, lint, ui};
use cmt::commands::{amend, completions, help, init, log, status, types_cmd, uninstall};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = args.first().cloned().unwrap_or_else(|| "commit".to_string());
    let rest: Vec<String> = args.iter().skip(1).cloned().collect();

    match cmd.as_str() {
        "commit" | "c" => {
            let write_only = rest.iter().any(|a| a == "--write-only");
            let dry_run = rest.iter().any(|a| a == "--dry-run");
            commit::run(write_only, dry_run);
        }
        "amend" | "a" => amend::run(&rest),
        "lint" | "l" => run_lint(&rest),
        "init" => init::run(&rest),
        "log" => log::run(&rest),
        "status" | "s" => status::run(&rest),
        "types" | "t" => types_cmd::run(&rest),
        "completions" => completions::run(&rest),
        "uninstall" => uninstall::run(&rest),
        "version" | "-v" | "--version" => help::print_version(),
        _ => help::print_help(),
    }
}

fn run_lint(args: &[String]) {
    let p = ui::palette();
    let msg = if let Some(file) = args.first() {
        match std::fs::read_to_string(file) {
            Ok(s) => s,
            Err(_) => {
                eprintln!("  {}✖{}  File not found: {}", p.red, p.reset, file);
                std::process::exit(1);
            }
        }
    } else {
        let mut s = String::new();
        let _ = std::io::stdin().read_to_string(&mut s);
        s
    };
    let cfg = config::load();
    let result = lint::check(&msg, &cfg);
    print!("{}", lint::render(&result, &p));
    std::process::exit(result.exit_code());
}
```

- [ ] **Step 4: Ensure `src/lib.rs` exposes everything**

```rust
pub mod types;
pub mod config;
pub mod ui;
pub mod git;
pub mod lint;
pub mod picker;
pub mod commit;
pub mod hooks;
pub mod commands;
```

- [ ] **Step 5: Run full test suite**

Run: `cargo test`
Expected: PASS — all parity, integration, and smoke tests green.

- [ ] **Step 6: Manual end-to-end on the dev machine**

Run: in a scratch git repo, `git add` a file, then `cmt init`, `git commit` (hook fires picker), `cmt log 5`, `cmt lint <<< 'feat: x'`.
Expected: wizard works with arrows; commit succeeds; log renders; lint passes.

- [ ] **Step 7: Commit**

```bash
git add src/main.rs src/lib.rs tests/cli_smoke.rs
git commit -m "feat: wire main command dispatch"
```

---

## Task 16: Release CI — cross-compiled binaries

**Files:**
- Create: `.github/workflows/release.yml`

**Interfaces:**
- Consumes: built binary.
- Produces: per-target binaries attached to a GitHub Release on tag push.

- [ ] **Step 1: Write `.github/workflows/release.yml`**

```yaml
name: Release
on:
  push:
    tags: ["v*"]
permissions:
  contents: write
jobs:
  build:
    strategy:
      fail-fast: false
      matrix:
        include:
          - { os: ubuntu-latest,  target: x86_64-unknown-linux-gnu,  ext: "" }
          - { os: ubuntu-latest,  target: aarch64-unknown-linux-gnu, ext: "" }
          - { os: macos-latest,   target: x86_64-apple-darwin,       ext: "" }
          - { os: macos-latest,   target: aarch64-apple-darwin,      ext: "" }
          - { os: windows-latest, target: x86_64-pc-windows-msvc,    ext: ".exe" }
          - { os: windows-latest, target: aarch64-pc-windows-msvc,   ext: ".exe" }
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - name: Install cross linker (linux arm64)
        if: matrix.target == 'aarch64-unknown-linux-gnu'
        run: |
          sudo apt-get update
          sudo apt-get install -y gcc-aarch64-linux-gnu
          echo "CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc" >> "$GITHUB_ENV"
      - run: cargo build --release --target ${{ matrix.target }}
      - name: Stage artifact
        shell: bash
        run: |
          mkdir -p dist
          cp "target/${{ matrix.target }}/release/cmt${{ matrix.ext }}" "dist/cmt-${{ matrix.target }}${{ matrix.ext }}"
      - uses: softprops/action-gh-release@v2
        with:
          files: dist/cmt-${{ matrix.target }}${{ matrix.ext }}
```

- [ ] **Step 2: Validate workflow syntax**

Run: `cargo build --release` locally (confirms release profile builds).
Expected: a `target/release/cmt` binary is produced. (The matrix itself is verified on the first tag push.)

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/release.yml
git commit -m "ci: build and publish cross-compiled release binaries"
```

---

## Task 17: curl / PowerShell installers

**Files:**
- Modify: `install.sh` (replace bash-copy with binary download)
- Create: `install.ps1`

**Interfaces:**
- Consumes: GitHub Releases assets named `cmt-<target>[.exe]`.
- Produces: a `cmt` binary on the user's PATH.

- [ ] **Step 1: Rewrite `install.sh`**

```sh
#!/usr/bin/env sh
# Install the latest cmt release binary into ~/.local/bin.
set -eu
REPO="mihai-ro/cmt"
BIN_DIR="${CMT_BIN_DIR:-$HOME/.local/bin}"

os="$(uname -s)"
arch="$(uname -m)"
case "$os" in
  Linux)  plat="unknown-linux-gnu" ;;
  Darwin) plat="apple-darwin" ;;
  *) echo "Unsupported OS: $os" >&2; exit 1 ;;
esac
case "$arch" in
  x86_64|amd64) cpu="x86_64" ;;
  arm64|aarch64) cpu="aarch64" ;;
  *) echo "Unsupported arch: $arch" >&2; exit 1 ;;
esac

target="${cpu}-${plat}"
url="https://github.com/${REPO}/releases/latest/download/cmt-${target}"
mkdir -p "$BIN_DIR"
echo "Downloading cmt ($target)..."
curl -fsSL "$url" -o "$BIN_DIR/cmt"
chmod +x "$BIN_DIR/cmt"
echo "Installed to $BIN_DIR/cmt"
case ":$PATH:" in
  *":$BIN_DIR:"*) ;;
  *) echo "Add $BIN_DIR to your PATH." ;;
esac
```

- [ ] **Step 2: Verify install.sh shell-parses**

Run: `sh -n install.sh`
Expected: no output, exit 0.

- [ ] **Step 3: Write `install.ps1`**

```powershell
# Install the latest cmt release binary on Windows.
$ErrorActionPreference = "Stop"
$repo = "mihai-ro/cmt"
$binDir = if ($env:CMT_BIN_DIR) { $env:CMT_BIN_DIR } else { "$env:LOCALAPPDATA\Programs\cmt" }

$arch = if ([Environment]::Is64BitOperatingSystem) {
  if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64") { "aarch64" } else { "x86_64" }
} else { throw "Unsupported architecture" }

$target = "$arch-pc-windows-msvc"
$url = "https://github.com/$repo/releases/latest/download/cmt-$target.exe"
New-Item -ItemType Directory -Force -Path $binDir | Out-Null
$dest = Join-Path $binDir "cmt.exe"
Write-Host "Downloading cmt ($target)..."
Invoke-WebRequest -Uri $url -OutFile $dest
Write-Host "Installed to $dest"
Write-Host "Add $binDir to your PATH if it is not already."
```

- [ ] **Step 4: Verify PowerShell parses (if pwsh available; otherwise visual review)**

Run: `pwsh -NoProfile -Command "[scriptblock]::Create((Get-Content -Raw install.ps1)) | Out-Null; 'ok'"` (skip if pwsh not installed)
Expected: prints `ok`.

- [ ] **Step 5: Commit**

```bash
git add install.sh install.ps1
git commit -m "feat: binary-download installers for unix and windows"
```

---

## Task 18: npm wrapper (optionalDependencies)

**Files:**
- Modify: `package.json` (root → wrapper that resolves a platform binary)
- Create: `npm/bin/cmt.js` (launcher)
- Create: `npm/postinstall.js` (optional fallback download)
- Create: `npm/platform/*/package.json` (per-platform stub packages)
- Modify: `.npmignore` / `files`

**Interfaces:**
- Consumes: GitHub Release binaries (and/or per-platform npm packages).
- Produces: `npm i -g @mihairo/cmt` installs a working `cmt` with no Node needed at runtime (Node only launches the binary).

Approach (esbuild pattern): publish one package per platform containing the prebuilt binary; the main package lists them under `optionalDependencies`; npm installs only the matching one; the launcher resolves and execs it. A `postinstall.js` download fallback covers environments where optional deps were skipped.

- [ ] **Step 1: Update root `package.json`**

```json
{
  "name": "@mihairo/cmt",
  "version": "2.0.0",
  "description": "Conventional Commits CLI — a single native binary, zero runtime deps.",
  "keywords": ["conventional-commits", "commit", "git", "lint", "cli", "hook", "commitlint", "husky"],
  "homepage": "https://github.com/mihai-ro/cmt",
  "bugs": { "url": "https://github.com/mihai-ro/cmt/issues" },
  "repository": { "type": "git", "url": "git+https://github.com/mihai-ro/cmt.git" },
  "license": "MIT",
  "bin": { "cmt": "npm/bin/cmt.js" },
  "scripts": { "postinstall": "node npm/postinstall.js" },
  "files": ["npm/bin/cmt.js", "npm/postinstall.js", "schema/cmt.schema.json", "README.md", "LICENSE"],
  "optionalDependencies": {
    "@mihairo/cmt-linux-x64": "2.0.0",
    "@mihairo/cmt-linux-arm64": "2.0.0",
    "@mihairo/cmt-darwin-x64": "2.0.0",
    "@mihairo/cmt-darwin-arm64": "2.0.0",
    "@mihairo/cmt-win32-x64": "2.0.0",
    "@mihairo/cmt-win32-arm64": "2.0.0"
  }
}
```

- [ ] **Step 2: Write `npm/bin/cmt.js` launcher**

```js
#!/usr/bin/env node
"use strict";
const { spawnSync } = require("node:child_process");
const path = require("node:path");
const fs = require("node:fs");

function platformPkg() {
  const { platform, arch } = process;
  const map = {
    "linux-x64": "@mihairo/cmt-linux-x64",
    "linux-arm64": "@mihairo/cmt-linux-arm64",
    "darwin-x64": "@mihairo/cmt-darwin-x64",
    "darwin-arm64": "@mihairo/cmt-darwin-arm64",
    "win32-x64": "@mihairo/cmt-win32-x64",
    "win32-arm64": "@mihairo/cmt-win32-arm64",
  };
  return map[`${platform}-${arch}`];
}

function binName() {
  return process.platform === "win32" ? "cmt.exe" : "cmt";
}

function resolveBinary() {
  const pkg = platformPkg();
  if (pkg) {
    try {
      return require.resolve(`${pkg}/bin/${binName()}`);
    } catch (_) { /* fall through to local download */ }
  }
  const local = path.join(__dirname, binName());
  if (fs.existsSync(local)) return local;
  return null;
}

const bin = resolveBinary();
if (!bin) {
  console.error("cmt: no prebuilt binary found for this platform. Run: node npm/postinstall.js");
  process.exit(1);
}
const res = spawnSync(bin, process.argv.slice(2), { stdio: "inherit" });
process.exit(res.status === null ? 1 : res.status);
```

- [ ] **Step 3: Write `npm/postinstall.js` fallback download**

```js
"use strict";
const fs = require("node:fs");
const path = require("node:path");
const https = require("node:https");

function target() {
  const { platform, arch } = process;
  const cpu = arch === "arm64" ? "aarch64" : "x86_64";
  if (platform === "linux") return `${cpu}-unknown-linux-gnu`;
  if (platform === "darwin") return `${cpu}-apple-darwin`;
  if (platform === "win32") return `${cpu}-pc-windows-msvc`;
  return null;
}

function binName() {
  return process.platform === "win32" ? "cmt.exe" : "cmt";
}

// If the platform optional dependency resolved, nothing to do.
try {
  const map = {
    "linux-x64": "@mihairo/cmt-linux-x64", "linux-arm64": "@mihairo/cmt-linux-arm64",
    "darwin-x64": "@mihairo/cmt-darwin-x64", "darwin-arm64": "@mihairo/cmt-darwin-arm64",
    "win32-x64": "@mihairo/cmt-win32-x64", "win32-arm64": "@mihairo/cmt-win32-arm64",
  };
  const pkg = map[`${process.platform}-${process.arch}`];
  if (pkg) { require.resolve(`${pkg}/bin/${binName()}`); process.exit(0); }
} catch (_) { /* download fallback below */ }

const t = target();
if (!t) process.exit(0);
const ver = require("./../package.json").version;
const url = `https://github.com/mihai-ro/cmt/releases/download/v${ver}/cmt-${t}${process.platform === "win32" ? ".exe" : ""}`;
const dest = path.join(__dirname, "bin", binName());

function download(u, file, redirects = 0) {
  https.get(u, (res) => {
    if ([301, 302, 307, 308].includes(res.statusCode) && res.headers.location && redirects < 5) {
      return download(res.headers.location, file, redirects + 1);
    }
    if (res.statusCode !== 200) { console.error(`cmt: download failed (${res.statusCode})`); process.exit(0); }
    const out = fs.createWriteStream(file, { mode: 0o755 });
    res.pipe(out);
    out.on("finish", () => out.close());
  }).on("error", () => process.exit(0));
}
download(url, dest);
```

- [ ] **Step 4: Create one per-platform stub `package.json` (repeat for all six)**

`npm/platform/linux-x64/package.json`:

```json
{
  "name": "@mihairo/cmt-linux-x64",
  "version": "2.0.0",
  "os": ["linux"],
  "cpu": ["x64"],
  "files": ["bin/cmt"]
}
```

(Release CI copies the matching binary into each platform package's `bin/` before `npm publish`. Repeat the file for `linux-arm64`/`cpu:["arm64"]`, `darwin-x64`/`os:["darwin"]`, `darwin-arm64`, `win32-x64`/`os:["win32"]`+`bin/cmt.exe`, `win32-arm64`.)

- [ ] **Step 5: Verify the launcher resolves locally**

Run: `cp target/release/cmt npm/bin/cmt && node npm/bin/cmt.js --version`
Expected: prints `cmt version 2.0.0`. Then `rm npm/bin/cmt`.

- [ ] **Step 6: Commit**

```bash
git add package.json npm/ .npmignore
git commit -m "feat: npm wrapper with per-platform binary resolution"
```

---

## Task 19: Homebrew formula

**Files:**
- Create: `HomebrewFormula/cmt.rb` in this repo (the tap repo's formula mirrors it; CI bumps the tap).

**Interfaces:**
- Consumes: GitHub Release binaries.
- Produces: `brew install cmt` fetches the release binary instead of building bash.

- [ ] **Step 1: Write `HomebrewFormula/cmt.rb`**

```ruby
class Cmt < Formula
  desc "Conventional Commits CLI — a single native binary"
  homepage "https://github.com/mihai-ro/cmt"
  version "2.0.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/mihai-ro/cmt/releases/download/v#{version}/cmt-aarch64-apple-darwin"
      sha256 "REPLACE_WITH_ARM64_DARWIN_SHA"
    end
    on_intel do
      url "https://github.com/mihai-ro/cmt/releases/download/v#{version}/cmt-x86_64-apple-darwin"
      sha256 "REPLACE_WITH_X64_DARWIN_SHA"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/mihai-ro/cmt/releases/download/v#{version}/cmt-aarch64-unknown-linux-gnu"
      sha256 "REPLACE_WITH_ARM64_LINUX_SHA"
    end
    on_intel do
      url "https://github.com/mihai-ro/cmt/releases/download/v#{version}/cmt-x86_64-unknown-linux-gnu"
      sha256 "REPLACE_WITH_X64_LINUX_SHA"
    end
  end

  def install
    bin.install Dir["cmt-*"].first => "cmt"
  end

  test do
    assert_match "cmt version", shell_output("#{bin}/cmt --version")
  end
end
```

- [ ] **Step 2: Verify Ruby syntax**

Run: `ruby -c HomebrewFormula/cmt.rb`
Expected: `Syntax OK`.

- [ ] **Step 3: Note on CI bump**

The existing tap-bump workflow must be updated to (a) compute each asset's sha256 after release and (b) write them into the tap formula. Document this in the workflow PR; the SHAs are filled by CI, never by hand.

- [ ] **Step 4: Commit**

```bash
git add HomebrewFormula/cmt.rb
git commit -m "feat: homebrew formula for release binary"
```

---

## Task 20: Docs + README + version + release

**Files:**
- Modify: `README.md`
- Modify: `CONTRIBUTING.md`
- Modify: `release-please-config.json` / `.release-please-manifest.json` (switch to Rust + npm release flow)
- Modify: `.github/` release-please config as needed.

**Interfaces:**
- Consumes: everything above.
- Produces: accurate public docs and a 2.0.0 release path.

- [ ] **Step 1: Update README install section**

Replace the "zero dependencies, 1300 lines of bash" framing with "a single native binary." Update install blocks:

```markdown
## Install

**curl (any OS)**
```bash
curl -fsSL https://raw.githubusercontent.com/mihai-ro/cmt/main/install.sh | sh
```

**Windows (PowerShell)**
```powershell
irm https://raw.githubusercontent.com/mihai-ro/cmt/main/install.ps1 | iex
```

**npm**
```bash
npm install -g @mihairo/cmt
```

**Homebrew**
```bash
brew install mihairo/tap/cmt
```
```

Keep the Usage, Config, Hooks, Lint rules, and comparison sections; update the comparison table "one bash script" rows to "one native binary."

- [ ] **Step 2: Update CONTRIBUTING.md for the Rust toolchain**

Document: `cargo build`, `cargo test`, `cargo fmt`, `cargo clippy -- -D warnings`; the parity fixtures in `tests/fixtures/`; how to regenerate a fixture deliberately (and that doing so is a reviewed behavior change).

- [ ] **Step 3: Verify docs build-free correctness**

Run: `cargo test` (ensures examples/paths referenced still valid) and visually confirm README commands match implemented flags.
Expected: tests pass; no stale flags in README.

- [ ] **Step 4: Bump version + changelog entry**

Ensure `Cargo.toml` `version = "2.0.0"` and root `package.json` `"version": "2.0.0"` agree. Add a `CHANGELOG.md` entry summarizing the native-binary rewrite as a breaking distribution change (config/CLI unchanged).

- [ ] **Step 5: Commit**

```bash
git add README.md CONTRIBUTING.md CHANGELOG.md release-please-config.json .release-please-manifest.json
git commit -m "docs: update for native binary v2.0.0"
```

- [ ] **Step 6: Final full verification**

Run: `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test`
Expected: all green. This is the gate before tagging `v2.0.0`.

---

## Self-Review

**Spec coverage check (spec section → task):**
- Architecture / module split → Tasks 1, 3–15 (file structure matches spec §1).
- Picker / arrow nav → Tasks 8 (state machine) + 9 (crossterm render); Windows guarantee via CI matrix in Task 1 + manual matrix in Task 9.
- CLI parity surface → Tasks 10 (commit), 13 (log/types/status/amend), 12 (init/uninstall), 14 (completions/help/version), 15 (dispatch + lint + aliases + default command).
- Config parity → Task 4 (serde load + canonical dump) with fixtures from Task 2.
- Lint parity (8 rules) → Task 6 with fixtures from Task 2.
- Hooks parity → Task 11 (snippets/marker blocks) + Task 12 (init/uninstall wiring) with fixtures from Task 2.
- UI/colors parity → Task 5 (palette/box/wordwrap), honored across command outputs.
- Testing strategy (snapshots + state-machine units + CI matrix) → Task 2 (capture) + per-task tests + Task 1 (CI).
- Distribution (releases, curl/ps1, npm, homebrew) → Tasks 16, 17, 18, 19. crates.io intentionally excluded per spec non-goals.
- Migration/version 2.0.0 → Task 20.

**Placeholder scan:** Homebrew SHAs are `REPLACE_WITH_*` by design (filled by CI post-release, never by hand — noted in Task 19 Step 3). No other TODO/TBD placeholders; every code step contains complete code.

**Type consistency:** `Config`/`Rules`/`CommitType` field names are consistent across Tasks 3, 4, 6, 10, 13. `Draft` fields match between Task 10 definition and Task 10/13 use. `picker::select` signature `(prompt, items) -> Option<usize>` consistent across Tasks 9, 10. `hooks::Outcome` variants consistent across Tasks 11, 12. `dump_canonical` format in Task 4 matches the `.resolved` fixtures written in Task 2 Step 5.

**Known coupling to flag for the implementer:** several tasks (12, 13, 15) reference the binary via `CARGO_BIN_EXE_cmt`, which only exists once `main.rs` dispatch (Task 15) is in place — integration tests in Tasks 12 are marked to run after Task 15. The plan's commit order keeps each task's *unit* tests self-contained; cross-binary integration tests finalize at Task 15.
