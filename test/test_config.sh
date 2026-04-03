#!/usr/bin/env bash
# load_config parser tests

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/helpers.sh"
CMT_SOURCED=1 source "$SCRIPT_DIR/../cmt"
set +euo pipefail  # cmt sets strict mode; disable for test execution

_tmpjson() { mktemp "${TMPDIR:-/tmp}/cmt-test-XXXXXX"; }

suite "config — defaults when no file"
CMT_CONFIG_FILE="/nonexistent/.cmt.json" load_config
assert_eq "default RULE_MAX_HEADER"    "72"   "$RULE_MAX_HEADER"
assert_eq "default RULE_REQUIRE_SCOPE" "0"    "$RULE_REQUIRE_SCOPE"
assert_eq "default RULE_DISALLOW_UPPER" "0"   "$RULE_DISALLOW_UPPER"
assert_eq "default RULE_DISALLOW_PERIOD" "0"  "$RULE_DISALLOW_PERIOD"
assert_eq "11 builtin types"           "11"   "${#TYPES[@]}"

suite "config — inline scopes"
cfg=$(_tmpjson)
cat > "$cfg" << 'JSON'
{
  "scopes": ["auth", "api", "ui"]
}
JSON
CMT_CONFIG_FILE="$cfg" load_config
assert_eq "3 scopes loaded"            "3"    "${#CUSTOM_SCOPES[@]}"
assert_eq "first scope"                "auth" "${CUSTOM_SCOPES[0]}"
assert_eq "last scope"                 "ui"   "${CUSTOM_SCOPES[2]}"
rm -f "$cfg"

suite "config — multiline scopes"
cfg=$(_tmpjson)
cat > "$cfg" << 'JSON'
{
  "scopes": [
    "auth",
    "api"
  ]
}
JSON
CMT_CONFIG_FILE="$cfg" load_config
assert_eq "2 multiline scopes"         "2"    "${#CUSTOM_SCOPES[@]}"
assert_eq "first multiline scope"      "auth" "${CUSTOM_SCOPES[0]}"
rm -f "$cfg"

suite "config — customTypes inline"
cfg=$(_tmpjson)
cat > "$cfg" << 'JSON'
{
  "customTypes": [
    {"type":"wip","emoji":"🚧","semver":"none","description":"WIP"}
  ]
}
JSON
CMT_CONFIG_FILE="$cfg" load_config
assert_eq "12 types (11 + 1 custom)"   "12"   "${#TYPES[@]}"
assert_eq "custom type key"            "wip"  "${TYPES[11]%%|*}"
rm -f "$cfg"

suite "config — rules overrides"
cfg=$(_tmpjson)
cat > "$cfg" << 'JSON'
{
  "rules": {
    "maxHeaderLength": 50,
    "requireScope": true,
    "disallowUpperCaseDescription": true,
    "disallowTrailingPeriod": true
  }
}
JSON
CMT_CONFIG_FILE="$cfg" load_config
assert_eq "maxHeaderLength 50"         "50"   "$RULE_MAX_HEADER"
assert_eq "requireScope true"          "1"    "$RULE_REQUIRE_SCOPE"
assert_eq "disallowUpper true"         "1"    "$RULE_DISALLOW_UPPER"
assert_eq "disallowPeriod true"        "1"    "$RULE_DISALLOW_PERIOD"
rm -f "$cfg"

summary
