#!/usr/bin/env bash
# One-time recorder: drives the current bash ./cmt to snapshot parity outputs.
# After this runs and fixtures are committed, the bash script is deleted.
# Note: NO_COLOR=1 triggers an unbound variable bug in the bash script (ACCENT_BOLD),
# so we capture colored output and strip ANSI codes via sed instead.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BASH_CMT="$ROOT/cmt"
FX="$ROOT/tests/fixtures"
mkdir -p "$FX/lint"

# Strip ANSI escape codes (BSD sed compatible)
strip_ansi() {
  sed $'s/\033\\[[0-9;]*m//g'
}

emit_lint() { # name, message
  local name="$1" msg="$2"
  printf '%s' "$msg" > "$FX/lint/$name.msg"
  set +e
  bash "$BASH_CMT" lint "$FX/lint/$name.msg" > "$FX/lint/$name.out.raw" 2>&1
  local code=$?
  strip_ansi < "$FX/lint/$name.out.raw" > "$FX/lint/$name.out"
  rm "$FX/lint/$name.out.raw"
  echo $code > "$FX/lint/$name.code"
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

echo "all fixtures captured"
