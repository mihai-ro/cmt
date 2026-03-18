# cmt

> Conventional Commits CLI — zero dependencies, one bash script.

[![npm](https://img.shields.io/npm/v/@mihairo/cmt?label=npm)](https://npmjs.com/package/@mihairo/cmt)
[![Conventional Commits](https://img.shields.io/badge/Conventional%20Commits-1.0.0-fe5196?logo=conventionalcommits)](https://conventionalcommits.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

---

## Install

**npm (any project — no Node required at runtime)**

```bash
npm install -g @mihairo/cmt
```

**Homebrew**

```bash
brew tap mihairo/tap
brew install cmt
```

**curl (no package manager)**

```bash
curl -fsSL https://raw.githubusercontent.com/mihai-ro/cmt/main/cmt \
  -o ~/.local/bin/cmt && chmod +x ~/.local/bin/cmt
```

---

## Usage

```
cmt <command> [flags]

  init [--husky] [--lint]   create .cmt.json + install git hook(s)
  commit                     interactive commit builder
  lint [file]                lint a message file or stdin  →  exit 1 on error
  log [n]                    pretty log of last n commits  (default: 20)
  types                      list available commit types
  uninstall                  remove cmt-managed hooks and .cmt.json
  version
```

**Set up a repo:**

```bash
cd my-project
cmt init                 # picker hook only
cmt init --lint          # picker + lint git commit -m "..." commits
cmt init --husky         # husky v9 format  (.husky/prepare-commit-msg)
cmt init --husky --lint  # both hooks, husky format
```

**Guided interactive commit:**

```bash
git add .
git commit     # triggers the interactive picker automatically
# or:
cmt commit     # run directly
```

**Lint from anywhere:**

```bash
echo "feat(api): add login" | cmt lint          # exit 0
echo "bad message"          | cmt lint          # exit 1

# lint every commit on a branch (CI)
git log --format="%s" origin/main..HEAD | while IFS= read -r msg; do
  echo "$msg" | cmt lint || exit 1
done
```

---

## Commit format

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

**Breaking change:**

```
feat!: drop support for Node 14
feat(api)!: redesign endpoints
BREAKING CHANGE: /auth now returns JWT
```

**Built-in types**

| Type       | Emoji | SemVer impact |
| ---------- | ----- | ------------- |
| `feat`     | ✨    | minor         |
| `fix`      | 🐛    | patch         |
| `docs`     | 📚    | —             |
| `style`    | 💅    | —             |
| `refactor` | ♻️    | —             |
| `perf`     | ⚡    | patch         |
| `test`     | 🧪    | —             |
| `build`    | 🏗️    | —             |
| `ci`       | 🔧    | —             |
| `chore`    | 🔩    | —             |
| `revert`   | ⏪    | patch         |

---

## Configuration — `.cmt.json`

`cmt init` creates `.cmt.json` at your repo root with a `$schema` pointer.
VS Code, JetBrains, and any JSON Language Server will autocomplete and
validate it automatically — no extension needed.

```json
{
  "$schema": "https://raw.githubusercontent.com/mihai-ro/cmt/main/schema/cmt.schema.json",
  "version": "1.0.0",
  "customTypes": [
    {
      "type": "wip",
      "emoji": "🚧",
      "semver": "none",
      "description": "Work in progress"
    },
    {
      "type": "security",
      "emoji": "🔒",
      "semver": "patch",
      "description": "Security fix"
    }
  ],
  "scopes": ["auth", "api", "ui", "db"],
  "rules": {
    "maxHeaderLength": 72,
    "requireScope": false,
    "allowBreakingChanges": ["feat", "fix"],
    "disallowUpperCaseDescription": false,
    "disallowTrailingPeriod": false
  }
}
```

**Scopes** — when the `scopes` array is non-empty, `cmt commit` shows an
arrow-key picker with your configured scopes, a "custom" option for free-text
entry, and a "skip" option. Leave `scopes` empty to always use free-text input.

Commit `.cmt.json` — your whole team shares the same types, scopes, and rules.

---

## Hooks

### `prepare-commit-msg` (always installed)

Intercepts plain `git commit`, runs the interactive picker, and writes the
message — so git never opens its editor. Skips amends, merges, squashes, and
any commit that already has a source message.

### `commit-msg` (opt-in via `--lint`)

Lints the final commit message. Catches `git commit -m "..."`, `--amend`,
and commits from GUI tools:

```bash
cmt init --lint
```

Both hooks follow the same append/create pattern — if a hook file already
exists they append a clearly-marked block rather than overwriting it.
`cmt uninstall` removes only the cmt blocks, leaving any other content intact.

---

## Integrations

### Husky v9

```bash
cmt init --husky --lint    # writes .husky/prepare-commit-msg + .husky/commit-msg
git add .husky/            # commit them — every teammate gets them on clone
```

### lint-staged

Add to `.husky/pre-commit`:

```sh
npx lint-staged
```

`cmt` handles commit message linting separately — the two hooks are completely independent.

### GitHub Actions

```yaml
- name: Lint commit messages
  run: |
    git log --format="%s" origin/main..HEAD | while IFS= read -r msg; do
      echo "$msg" | cmt lint || exit 1
    done
```

### Git operations and hook behaviour

| Operation             | Hook fires?      | Result                               |
| --------------------- | ---------------- | ------------------------------------ |
| `git commit`          | ✅               | picker runs                          |
| `git commit -m "..."` | ✅ with `--lint` | linted                               |
| `git commit --amend`  | ✅ with `--lint` | linted                               |
| `git merge --no-ff`   | ✅               | ⏭ skipped (auto-generated message)  |
| `git revert`          | ✅               | ⏭ skipped (auto-generated message)  |
| `fixup!` / `squash!`  | ✅               | ⏭ skipped                           |
| `git pull --rebase`   | ✅               | ✅ passes (replays existing commits) |
| Empty/aborted commit  | ✅               | ⏭ skipped                           |

---

## Lint rules

| Rule                                      | Config key                     | Default  | On fail    |
| ----------------------------------------- | ------------------------------ | -------- | ---------- |
| Header format `type(scope)?: description` | —                              | required | ❌ error   |
| Valid type                                | —                              | required | ❌ error   |
| Non-empty description                     | —                              | required | ❌ error   |
| Blank line before body                    | —                              | required | ❌ error   |
| Scope required                            | `requireScope`                 | `false`  | ❌ error   |
| Uppercase description                     | `disallowUpperCaseDescription` | `false`  | ⚠️ / ❌    |
| Trailing period                           | `disallowTrailingPeriod`       | `false`  | ⚠️ / ❌    |
| Header length                             | `maxHeaderLength`              | `72`     | ⚠️ warning |

Warnings exit `0`. Errors exit `1`.

---

## Why not commitlint + husky + commitizen?

|                            | **cmt**            | commitlint + husky | commitizen      |
| -------------------------- | ------------------ | ------------------ | --------------- |
| Runtime dependencies       | **0**              | ~15 npm packages   | Python + pip    |
| Works in any language repo | ✅                 | ❌ needs Node      | ❌ needs Python |
| Install                    | copy one file      | `npm install`      | `pip install`   |
| Interactive commit prompt  | ✅                 | via cz-commitlint  | ✅              |
| Commit-msg linting         | ✅ opt-in `--lint` | ✅                 | ✅              |
| Husky v9 support           | ✅                 | native             | via config      |
| JSON Schema / intellisense | ✅                 | partial            | ❌              |
| Custom types + scopes      | ✅ `.cmt.json`     | ✅                 | ✅              |

---

## License

MIT
