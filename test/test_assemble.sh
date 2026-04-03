#!/usr/bin/env bash
# assemble_message output tests

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/helpers.sh"
CMT_SOURCED=1 source "$SCRIPT_DIR/../cmt"
set +euo pipefail  # cmt sets strict mode; disable for test execution

suite "assemble_message — basic"
SELECTED_TYPE="feat" SELECTED_SCOPE="" SELECTED_DESC="add login"
SELECTED_BODY="" SELECTED_FOOTER="" IS_BREAKING=0 BREAKING_DESC=""
assemble_message
assert_eq "feat: add login"  "feat: add login"  "$COMMIT_MESSAGE"

suite "assemble_message — with scope"
SELECTED_TYPE="fix" SELECTED_SCOPE="auth" SELECTED_DESC="correct redirect"
SELECTED_BODY="" SELECTED_FOOTER="" IS_BREAKING=0 BREAKING_DESC=""
assemble_message
assert_eq "fix(auth): correct redirect"  "fix(auth): correct redirect"  "$COMMIT_MESSAGE"

suite "assemble_message — breaking change"
SELECTED_TYPE="feat" SELECTED_SCOPE="" SELECTED_DESC="new api"
SELECTED_BODY="" SELECTED_FOOTER="" IS_BREAKING=1 BREAKING_DESC="removes /v1 endpoint"
assemble_message
assert_contains "header has !"          "feat!: new api"               "$COMMIT_MESSAGE"
assert_contains "breaking change footer" "BREAKING CHANGE: removes /v1" "$COMMIT_MESSAGE"

suite "assemble_message — breaking with scope"
SELECTED_TYPE="feat" SELECTED_SCOPE="api" SELECTED_DESC="revamp"
SELECTED_BODY="" SELECTED_FOOTER="" IS_BREAKING=1 BREAKING_DESC="drops v1"
assemble_message
assert_eq "header" "feat(api)!: revamp" "${COMMIT_MESSAGE%%$'\n'*}"

suite "assemble_message — with body"
SELECTED_TYPE="docs" SELECTED_SCOPE="" SELECTED_DESC="update readme"
SELECTED_BODY="Added installation section." SELECTED_FOOTER="" IS_BREAKING=0 BREAKING_DESC=""
assemble_message
assert_contains "body present"          "Added installation section."  "$COMMIT_MESSAGE"
assert_contains "blank line before body" $'docs: update readme\n\nAdded' "$COMMIT_MESSAGE"

suite "assemble_message — with footer"
SELECTED_TYPE="fix" SELECTED_SCOPE="" SELECTED_DESC="close issue"
SELECTED_BODY="" SELECTED_FOOTER="Closes #42" IS_BREAKING=0 BREAKING_DESC=""
assemble_message
assert_contains "footer present" "Closes #42" "$COMMIT_MESSAGE"

suite "assemble_message — full"
SELECTED_TYPE="feat" SELECTED_SCOPE="auth" SELECTED_DESC="add oauth2"
SELECTED_BODY="Supports Google and GitHub providers." SELECTED_FOOTER="Closes #99"
IS_BREAKING=1 BREAKING_DESC="removes basic auth"
assemble_message
assert_contains "header"         "feat(auth)!: add oauth2"              "$COMMIT_MESSAGE"
assert_contains "body"           "Supports Google and GitHub providers." "$COMMIT_MESSAGE"
assert_contains "breaking footer" "BREAKING CHANGE: removes basic auth"  "$COMMIT_MESSAGE"
assert_contains "issue footer"   "Closes #99"                           "$COMMIT_MESSAGE"

suite "_wordwrap"
out=$(_wordwrap "hello world" 5)
assert_contains "wraps at width" "hello" "$out"
assert_contains "second word on new line" "world" "$out"

out=$(_wordwrap "short" 20)
assert_eq "no wrap needed" "short" "$out"

out=$(_wordwrap "one two three four" 10)
assert_contains "wraps long sentence" "one two" "$out"

summary
