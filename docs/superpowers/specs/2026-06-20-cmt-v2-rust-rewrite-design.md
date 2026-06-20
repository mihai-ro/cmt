# cmt v2 — Native Rust Rewrite (Design)

Date: 2026-06-20
Status: Approved (pending spec review)
Target version: 2.0.0

## Summary

Replace the ~1300-line bash `cmt` script with a single, small, native Rust
binary. Same CLI, same `.cmt.json` config + schema, same hooks, same lint
behavior, same wizard UX — a clean drop-in replacement that runs natively on
Linux, macOS, and Windows (no bash, no Git-Bash, no WSL required).

The driving goal: arrow-key navigation in the commit wizard must work
identically on all three platforms, with zero exceptions. The current bash
picker relies on `stty`, `/dev/tty`, and hand-rolled ANSI — fragile on Windows
terminals (mintty/winpty/CRLF). Moving to `crossterm`, which talks to the
Windows Console API natively, eliminates that entire class of bug.

This is a clean break: the bash script is removed. No legacy script is kept.

## Goals

- Single static binary per OS/arch. Super light (~1–2MB), instant startup.
- Native, identical arrow-key wizard on Linux/macOS/Windows.
- 1:1 behavioral parity with the current bash `cmt` (CLI, config, lint, hooks,
  output) so existing users and committed `.cmt.json` files keep working.
- Minimal dependency footprint.
- Strong contributor story: clear module boundaries, CI matrix across OSes.

## Non-Goals (deferred to 2.1+)

- New features (AI messages, staged-diff preview, richer wizard, etc.).
- `cargo` / crates.io distribution.
- Native cmd/PowerShell *scripting* support beyond running the binary.

## Language & Dependencies

- **Rust** (stable).
- **crossterm** — raw mode, key events, cursor control. The arrow-nav core;
  native Windows Console support is the whole reason for the rewrite.
- **serde** + **serde_json** — parse `.cmt.json` (replaces the bash JSON
  state-machine parser).
- Optionally **unicode-width** if emoji/box alignment needs it (decide during
  implementation; otherwise omit).

Explicitly avoided to stay lean and keep exact UX control:
- **clap** — hand-roll the tiny arg dispatcher to match current flags/UX exactly.
- **ratatui** — crossterm raw mode + manual render keeps the binary small and
  gives full control over the picker.

## Module Architecture

Each module has one job, small enough to reason about in isolation.

```
src/
  main.rs        # arg dispatch -> command functions
  config.rs      # load .cmt.json (serde_json), apply defaults, builtin types
  types.rs       # CommitType, Rules, Config structs
  commit.rs      # wizard flow + assemble_message
  picker.rs      # crossterm select widget (arrow-nav core)
  lint.rs        # lint rules + exit codes
  hooks.rs       # install/uninstall, snippets, husky, marker blocks
  log.rs         # pretty git log
  completions.rs # bash/zsh/fish completion strings
  ui.rs          # color palette, NO_COLOR/tty detection, commit box, wordwrap
  git.rs         # git subprocess invocations
```

### Module interfaces (contracts)

- `config::load() -> Config` — reads `CMT_CONFIG_FILE` (default `.cmt.json` in
  repo/cwd per current behavior), merges over defaults. Pure given a file path;
  test by pointing at fixtures.
- `picker::select(prompt, items) -> Option<usize>` — renders, handles keys,
  returns chosen index (original list). Internally split into a pure state
  machine (`PickerState`: filter/move/scroll) and a render/IO layer, so the
  navigation logic is unit-testable without a TTY. Falls back to a numbered
  prompt when raw mode is unavailable.
- `commit::run(opts)` — drives the wizard, calls `assemble_message`, then
  either commits, prints to stdout (`--write-only`), or prints only
  (`--dry-run`).
- `lint::check(msg, &Config) -> LintResult` — pure; returns errors/warnings.
  Caller maps to exit code (errors → 1, warnings-only → 0).
- `hooks::install/uninstall` — idempotent marker-block mutation of hook files.

## The Picker (critical path)

crossterm `event::read()` mapped to:

- `Up` / `Char('k')` → move up (wraps, adjusts scroll window)
- `Down` / `Char('j')` → move down (wraps, adjusts scroll window)
- printable char → append to incremental filter
- `Backspace` → remove last filter char
- `Esc` → clear filter (lone Esc)
- `Char('q')` → quit only when filter empty, else appends to filter
- `Enter` → confirm
- `Ctrl-C` → abort (exit 1)

Parity with current `_pick`:

- Incremental filter line (`/ type to filter` placeholder; `/ <filter>` active).
- 7-item scroll window with `↑ N more` / `↓ N more` hints.
- Cursor hidden during selection, restored after.
- Collapse to the single selected line on confirm.
- Numbered-menu fallback when raw mode can't be enabled.

State machine is rendering-independent and unit-tested: given a key sequence,
assert resulting `cur`/`top`/`filter`/visible set. Rendering uses relative
cursor moves (crossterm `MoveUp` / clear-line), matching the current
relative-cursor approach that fixed Terminal.app.

## CLI Parity Surface (must match 1:1)

Commands and flags:

- `init [--husky] [--lint]` — create `.cmt.json` + install hook(s).
- `commit [--write-only] [--dry-run]` — interactive builder.
  `--write-only` (used by the hook) prints message to stdout, no git commit;
  `--dry-run` prints assembled message, no commit.
- `lint [file]` — lint a file or stdin; exit 1 on error, 0 otherwise.
- `log [n]` — pretty log of last n commits (default 20).
- `types` — list available commit types.
- `amend` — amend helper (current behavior).
- `status` — current behavior.
- `uninstall` — remove cmt-managed hooks + `.cmt.json` blocks.
- `completions [bash|zsh|fish]` — emit completion script.
- `version`, `help`.

Wizard flow (exact order): type → scope → description → body → breaking →
footer → assemble → confirm `[Y/n/e]` (`e` opens `$EDITOR`, default `vi`;
ensure Windows-appropriate default editor handling).

`assemble_message` output (exact ordering):

```
type(scope)!: description
<blank>
body
<blank>
BREAKING CHANGE: <breaking desc>
<blank>
footer
```

(Optional segments omitted when empty; `!` only when breaking; `(scope)` only
when scope set.)

## Config Parity (`.cmt.json`)

Fields preserved exactly:

- `$schema` — unchanged schema file (`schema/cmt.schema.json`) ships as-is.
- `customTypes[]` — `{ type, emoji, semver, description }`, merged after the
  builtin types. Defaults for missing fields match current bash
  (`emoji: ⚙️`, `semver: none`, `description: Custom type`).
- `scopes[]` — non-empty → picker shows scopes + "custom" + "skip"; empty →
  free-text scope input.
- `rules`:
  - `maxHeaderLength` (default 72) — warning when exceeded.
  - `requireScope` (default false) — error when missing.
  - `disallowUpperCaseDescription` (default false) — error vs warning.
  - `disallowTrailingPeriod` (default false) — error vs warning.
  - `allowBreakingChanges[]` (default `["feat","fix"]`).
- `CMT_CONFIG_FILE` env override respected.

Builtin commit types: identical set to current bash.

## Lint Parity (8 rules)

Same rules, same messages, same exit semantics (warnings exit 0, errors exit 1):

1. Header format `type(scope)?: description`.
2. Valid type (builtin + custom).
3. Non-empty description.
4. Description capitalization (warn, or error if `disallowUpperCaseDescription`).
5. Trailing period (warn, or error if `disallowTrailingPeriod`).
6. Header length vs `maxHeaderLength` (warn).
7. Scope required (error if `requireScope`).
8. Blank line required between header and body (error).

Report formatting (icons, ordering, summary lines) matches current output.

## Hooks Parity

- `prepare-commit-msg` (always): runs `cmt commit --write-only > "$1"`; skips
  when `$2` is set (amend/merge/squash/message already present). POSIX `sh`
  snippet, unchanged, with binary-resolution probe list
  (`./node_modules/.bin/cmt`, `~/.local/bin/cmt`, `/usr/local/bin/cmt`,
  `/opt/homebrew/bin/cmt`, then `command -v cmt`).
- `commit-msg` (opt-in `--lint`): runs `cmt lint "$1"`; exits with lint code.
- Marker blocks `# >>> cmt` / `# <<< cmt` (and `cmt-lint`), append-or-replace
  via the same logic; `uninstall` strips only cmt blocks, leaving other content.
- Husky v9 paths: `.husky/prepare-commit-msg`, `.husky/commit-msg`.

Note: hook snippets remain POSIX `sh` (husky/git run them under sh) — they call
the native binary; no behavior change.

## UI / Colors Parity

- Same truecolor palette (accent cyan, green/yellow/red/blue, muted).
- TTY detection + `NO_COLOR` honored (no color when not a tty / NO_COLOR set).
- Commit-message box rendering + word-wrap at terminal width (matching
  `_print_commit_box` / `_wordwrap`), capped/min widths preserved.

## Testing Strategy

Parity is proven against **captured fixture snapshots**, not a live bash oracle
(the bash script is removed):

1. **Snapshot capture (one-time, pre-removal):** run the current bash `cmt`
   over a fixed set of inputs and record outputs as committed expected files:
   - lint: a corpus of messages → expected stdout + exit code.
   - assemble: selection inputs → expected message text.
   - config parse: a set of `.cmt.json` files → expected resolved config dump.
   - hooks: before/after hook-file contents for install/replace/uninstall.
2. **Rust tests assert against those snapshots.** Any intentional change is a
   deliberate snapshot update in a reviewed commit.
3. **Picker state machine:** unit tests over key sequences → expected
   `cur/top/filter/visible`, no TTY needed.
4. **CI matrix:** GitHub Actions `ubuntu-latest`, `macos-latest`,
   `windows-latest` — build + test on each. This is the real cross-platform
   guarantee, including Windows arrow-nav via crossterm.

## Distribution

- **GitHub Releases:** CI builds binaries on tag for
  `linux x64/arm64`, `macos x64/arm64`, `windows x64/arm64`, attaches to release.
- **curl:** rewrite `install.sh` to detect OS/arch and download the matching
  release binary to `~/.local/bin/cmt`. Add `install.ps1` for Windows.
- **npm:** `@mihairo/cmt` becomes a thin wrapper. Per-platform
  `optionalDependencies` packages each ship one prebuilt binary; the main
  package resolves the right one at install (esbuild/swc pattern). `npm i -g
  @mihairo/cmt` still works; no Node at runtime.
- **Homebrew:** formula points at the release binary instead of the bash script;
  reuse the existing auto-bump CI workflow.
- Keep release-please; first release is **2.0.0**.

## Migration & Risk

- Drop-in: identical CLI, configs, hooks, output. README gets a short "v2 is a
  native binary" note; install instructions updated for binary downloads.
- Risk areas to watch during implementation:
  - Emoji/box width alignment (consider `unicode-width`).
  - `$EDITOR` spawning on Windows (sensible default, e.g. `notepad` fallback).
  - Hook shebang/`sh` portability under husky on Windows (snippets stay POSIX
    sh; binary resolution probe covers npm/global installs).
  - npm optionalDependencies binary-resolution edge cases (CI-test the wrapper).

## Rollout Order (high level)

1. Scaffold Rust project + module skeleton + CI matrix.
2. Capture parity snapshots from current bash script, then remove the script.
3. Implement config, types, lint (pure, snapshot-tested) first.
4. Implement picker (state machine + crossterm render) — verify on all 3 OSes.
5. Implement commit wizard + assemble + commit/confirm.
6. Implement hooks, init, uninstall, log, types, status, amend, completions.
7. Wire distribution (releases, install scripts, npm wrapper, homebrew).
8. Docs + 2.0.0 release.
