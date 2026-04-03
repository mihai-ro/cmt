#!/usr/bin/env bash
# cmt test runner — sources all test_*.sh files and reports totals

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

TOTAL_PASS=0
TOTAL_FAIL=0

run_suite() {
  local file="$1"
  local name; name=$(basename "$file")
  printf "\n\033[1m%s\033[0m\n" "$name"
  # Each test file runs in a subshell so variables don't leak
  local out _exit=0
  out=$(bash "$file" 2>&1) || _exit=$?
  printf '%s\n' "$out"
  # Extract pass/fail counts from the summary line
  local p f
  p=$(printf '%s' "$out" | grep -oE '[0-9]+ passed' | grep -oE '[0-9]+' | head -1 || true)
  f=$(printf '%s' "$out" | grep -oE '[0-9]+ failed' | grep -oE '[0-9]+' | head -1 || true)
  [[ -z "$p" ]] && p=0
  [[ -z "$f" ]] && f=0
  TOTAL_PASS=$(( TOTAL_PASS + p ))
  TOTAL_FAIL=$(( TOTAL_FAIL + f ))
}

for _f in "$SCRIPT_DIR"/test_*.sh; do
  run_suite "$_f"
done

printf "\n\033[1m─────────────────────────────\033[0m\n"
if [[ $TOTAL_FAIL -eq 0 ]]; then
  printf "  \033[32m✔ All %d tests passed\033[0m\n\n" "$TOTAL_PASS"
else
  printf "  \033[32m%d passed\033[0m  \033[31m%d FAILED\033[0m\n\n" "$TOTAL_PASS" "$TOTAL_FAIL"
  exit 1
fi
