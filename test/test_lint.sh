#!/usr/bin/env bash
# Lint rule battery

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/helpers.sh"
CMT_SOURCED=1 source "$SCRIPT_DIR/../cmt"
set +euo pipefail  # cmt sets strict mode; disable for test execution

suite "lint — valid messages"
assert_exit "feat: add login"                            0 lint_message "feat: add login"
assert_exit "fix: correct typo"                         0 lint_message "fix: correct typo"
assert_exit "feat(auth): add oauth2 support"            0 lint_message "feat(auth): add oauth2 support"
assert_exit "feat!: breaking new api"                   0 lint_message "feat!: breaking new api"
assert_exit "feat(scope)!: breaking with scope"         0 lint_message "feat(scope)!: breaking with scope"
assert_exit "docs: update readme"                       0 lint_message "docs: update readme"
assert_exit "chore: update deps"                        0 lint_message "chore: update deps"
assert_exit "revert: undo login feature"                0 lint_message "revert: undo login feature"

suite "lint — valid with body"
assert_exit "feat with body and blank line" 0 lint_message $'feat: add feature\n\nThis is the body'
assert_exit "fix with multi-line body"      0 lint_message $'fix: patch it\n\nLine one\nLine two'

suite "lint — invalid messages"
assert_exit "empty string"                              1 lint_message ""
assert_exit "no colon separator"                        1 lint_message "feat add login"
assert_exit "unknown type"                              1 lint_message "unknown: do something"
assert_exit "empty description"                         1 lint_message "feat: "
assert_exit "no blank line before body"                 1 lint_message $'feat: title\nbody without blank line'
assert_exit "uppercase type"                            1 lint_message "FEAT: do something"

suite "lint — warnings only (exit 0)"
assert_exit "uppercase description (warning)"           0 lint_message "feat: Add something"
assert_exit "trailing period (warning)"                 0 lint_message "feat: add something."

suite "lint — rule: disallowUpperCaseDescription"
_cfg=$(mktemp "${TMPDIR:-/tmp}/cmt-lint-test-XXXXXX")
cat > "$_cfg" << 'JSON'
{
  "rules": {
    "disallowUpperCaseDescription": true
  }
}
JSON
_orig_cfg="$CMT_CONFIG_FILE"; CMT_CONFIG_FILE="$_cfg"
assert_exit "uppercase description becomes error"       1 lint_message "feat: Add something"
assert_exit "lowercase description still valid"         0 lint_message "feat: add something"
CMT_CONFIG_FILE="$_orig_cfg"; rm -f "$_cfg"

suite "lint — rule: disallowTrailingPeriod"
_cfg=$(mktemp "${TMPDIR:-/tmp}/cmt-lint-test-XXXXXX")
cat > "$_cfg" << 'JSON'
{
  "rules": {
    "disallowTrailingPeriod": true
  }
}
JSON
_orig_cfg="$CMT_CONFIG_FILE"; CMT_CONFIG_FILE="$_cfg"
assert_exit "trailing period becomes error"             1 lint_message "feat: add something."
assert_exit "no trailing period still valid"            0 lint_message "feat: add something"
CMT_CONFIG_FILE="$_orig_cfg"; rm -f "$_cfg"

suite "lint — rule: requireScope"
_cfg=$(mktemp "${TMPDIR:-/tmp}/cmt-lint-test-XXXXXX")
cat > "$_cfg" << 'JSON'
{
  "rules": {
    "requireScope": true
  }
}
JSON
_orig_cfg="$CMT_CONFIG_FILE"; CMT_CONFIG_FILE="$_cfg"
assert_exit "scope required — missing"                  1 lint_message "feat: add something"
assert_exit "scope required — present"                  0 lint_message "feat(api): add something"
CMT_CONFIG_FILE="$_orig_cfg"; rm -f "$_cfg"

suite "lint — rule: maxHeaderLength"
_cfg=$(mktemp "${TMPDIR:-/tmp}/cmt-lint-test-XXXXXX")
cat > "$_cfg" << 'JSON'
{
  "rules": {
    "maxHeaderLength": 30
  }
}
JSON
_orig_cfg="$CMT_CONFIG_FILE"; CMT_CONFIG_FILE="$_cfg"
assert_exit "header within limit"                       0 lint_message "feat: short"
CMT_CONFIG_FILE="$_orig_cfg"; rm -f "$_cfg"

summary
