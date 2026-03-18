# Local development & testing

## 1. Clone and make it available

```bash
git clone https://github.com/mihai-ro/cmt
cd cmt
```

**Option A — symlink (fastest)**

```bash
ln -sf "$PWD/cmt" ~/.local/bin/cmt
cmt version
```

**Option B — npm link**

```bash
npm link
cmt version

# When done:
npm unlink -g @mihairo/cmt
```

**Option C — direct path**

```bash
export PATH="$PWD:$PATH"
cmt version
```

---

## 2. Try it in a real repo

```bash
mkdir /tmp/cmt-test && cd /tmp/cmt-test
git init && git config user.email "you@example.com" && git config user.name "You"

cmt init              # creates .cmt.json + installs prepare-commit-msg hook
cmt init --lint       # also installs commit-msg linter
cmt types             # see all types
cmt commit            # interactive prompt — stage something first

# Test the hooks fire:
echo "hello" > f.txt && git add f.txt
git commit -m "bad message"             # blocked ✖  (with --lint)
git commit -m "feat: add hello file"    # passes ✔
git commit                              # opens interactive picker
```

---

## 3. Run the lint battery manually

```bash
# valid — should all exit 0:
echo "feat: add login"              | ./cmt lint
echo "fix(auth): handle null"       | ./cmt lint
echo "feat!: drop Node 14"          | ./cmt lint

# invalid — should all exit 1:
echo "bad message"                  | ./cmt lint
echo "FEAT: uppercase"              | ./cmt lint
echo "unknown: bad type"            | ./cmt lint

# warnings — exit 0 but prints ⚠:
echo "feat: Add something."         | ./cmt lint
```

---

## 4. Test hook install/uninstall

```bash
cd /tmp/cmt-test

# picker only
./cmt init
cat .git/hooks/prepare-commit-msg   # should have >>> cmt block

# with lint
./cmt init --lint
cat .git/hooks/commit-msg           # should have >>> cmt-lint block

# re-init is idempotent (updates block in place, no duplication)
./cmt init --lint
grep -c ">>> cmt-lint" .git/hooks/commit-msg   # should print 1

# append to existing hook
printf '#!/usr/bin/env bash\nnpx lint-staged\n' > .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
./cmt init
cat .git/hooks/pre-commit           # lint-staged still there, cmt block appended

# uninstall removes only cmt blocks, leaves other content
./cmt uninstall
cat .git/hooks/pre-commit           # lint-staged still there
ls .git/hooks/prepare-commit-msg 2>/dev/null || echo "removed ✔"
ls .cmt.json 2>/dev/null || echo "removed ✔"
```

---

## 5. Test husky mode

```bash
cd /tmp/cmt-test
./cmt init --husky --lint
cat .husky/prepare-commit-msg   # no shebang (husky v9 format)
cat .husky/commit-msg           # no shebang (husky v9 format)
```

---

## 6. Verify hook skips auto-generated messages

```bash
cd /tmp/cmt-test  # has cmt init --lint run

git commit --allow-empty -m "Merge branch 'feature'"
git commit --allow-empty -m 'Revert "feat: something"'
git commit --allow-empty -m "fixup! feat: add hello file"
git commit --allow-empty -m "squash! feat: add hello file"
# all should succeed (hook skips them)
```

---

## 7. Dry-run npm publish

```bash
npm pack --dry-run
# should include: cmt, schema/cmt.schema.json, README.md, LICENSE, package.json
```

---

## 8. Validate the JSON schema

```bash
npm install -g ajv-cli
ajv validate -s schema/cmt.schema.json -d .cmt.json
```
