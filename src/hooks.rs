use std::io;
use std::path::Path;

#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    Created,
    Appended,
    Updated,
    Removed,
    RemovedFile,
    NoOp,
}

pub fn prepare_snippet() -> String {
    // Byte-identical to fixtures/hooks/prepare_fresh.after minus the shebang line.
    concat!(
        "# >>> cmt \u{2014} Conventional Commits CLI\n",
        "[ -n \"${2:-}\" ] && exit 0\n",
        "_cmt_bin=\"\"\n",
        "for _p in \"./node_modules/.bin/cmt\" \"$HOME/.local/bin/cmt\" \"/usr/local/bin/cmt\" \"/opt/homebrew/bin/cmt\"; do\n",
        "  [ -x \"$_p\" ] && { _cmt_bin=\"$_p\"; break; }\n",
        "done\n",
        "[ -z \"$_cmt_bin\" ] && _cmt_bin=$(command -v cmt 2>/dev/null)\n",
        "[ -n \"$_cmt_bin\" ] && { \"$_cmt_bin\" commit --write-only > \"$1\"; exit 0; }\n",
        "exit 0\n",
        "# <<< cmt\n",
    )
    .to_string()
}

pub fn lint_snippet() -> String {
    // Byte-identical to fixtures/hooks/commitmsg_fresh.after minus the shebang line.
    concat!(
        "# >>> cmt-lint \u{2014} Conventional Commits CLI\n",
        "_cmt_bin=\"\"\n",
        "for _p in \"./node_modules/.bin/cmt\" \"$HOME/.local/bin/cmt\" \"/usr/local/bin/cmt\" \"/opt/homebrew/bin/cmt\"; do\n",
        "  [ -x \"$_p\" ] && { _cmt_bin=\"$_p\"; break; }\n",
        "done\n",
        "[ -z \"$_cmt_bin\" ] && _cmt_bin=$(command -v cmt 2>/dev/null)\n",
        "[ -n \"$_cmt_bin\" ] && { \"$_cmt_bin\" lint \"$1\"; exit $?; }\n",
        "exit 0\n",
        "# <<< cmt-lint\n",
    )
    .to_string()
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
    {
        let _ = path;
    }
}

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

pub fn install_block(
    path: &Path,
    snippet: &str,
    marker: &str,
    add_shebang: bool,
) -> io::Result<Outcome> {
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
    // Trailing space so "cmt" marker doesn't match "cmt-lint"
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
