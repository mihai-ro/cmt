#!/usr/bin/env bash
# Test helpers for cmt test suite

PASS=0
FAIL=0
_SUITE=""

suite() {
  _SUITE="$1"
  printf "\n  ${BOLD}%s${RESET}\n" "$_SUITE" 2>/dev/null || printf "\n  %s\n" "$_SUITE"
}

assert_eq() {
  local desc="$1" exp="$2" act="$3"
  if [[ "$exp" == "$act" ]]; then
    printf "  \033[32m✔\033[0m %s\n" "$desc"
    PASS=$(( PASS + 1 ))
  else
    printf "  \033[31m✖\033[0m %s\n    expected: %s\n    actual:   %s\n" "$desc" "$exp" "$act"
    FAIL=$(( FAIL + 1 ))
  fi
}

assert_contains() {
  local desc="$1" needle="$2" haystack="$3"
  if [[ "$haystack" == *"$needle"* ]]; then
    printf "  \033[32m✔\033[0m %s\n" "$desc"
    PASS=$(( PASS + 1 ))
  else
    printf "  \033[31m✖\033[0m %s\n    expected to contain: %s\n    actual: %s\n" "$desc" "$needle" "$haystack"
    FAIL=$(( FAIL + 1 ))
  fi
}

assert_exit() {
  local desc="$1" code="$2"; shift 2
  local r=0
  "$@" >/dev/null 2>&1 || r=$?
  if [[ $r -eq $code ]]; then
    printf "  \033[32m✔\033[0m %s\n" "$desc"
    PASS=$(( PASS + 1 ))
  else
    printf "  \033[31m✖\033[0m %s\n    expected exit: %s\n    actual exit:   %s\n" "$desc" "$code" "$r"
    FAIL=$(( FAIL + 1 ))
  fi
}

summary() {
  printf "\n"
  if [[ $FAIL -eq 0 ]]; then
    printf "  \033[32m✔ %d passed\033[0m\n\n" "$PASS"
  else
    printf "  \033[32m%d passed\033[0m  \033[31m%d failed\033[0m\n\n" "$PASS" "$FAIL"
    return 1
  fi
}
