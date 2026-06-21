pub fn bash() -> String {
    r#"# cmt bash completions — add to ~/.bashrc:  eval "$(cmt completions bash)"
_cmt_completions() {
  local cur="${COMP_WORDS[COMP_CWORD]}"
  local cmds="commit lint init log types status amend uninstall completions version help"
  COMPREPLY=( $(compgen -W "$cmds" -- "$cur") )
}
complete -F _cmt_completions cmt
"#
    .to_string()
}

pub fn zsh() -> String {
    r#"# cmt zsh completions — add to ~/.zshrc:  eval "$(cmt completions zsh)"
_cmt() {
  local -a cmds
  cmds=(
    'commit:Interactive commit builder'
    'lint:Lint a commit message file or stdin'
    'init:Set up repo with hook and config'
    'log:Pretty log of recent commits'
    'types:List available commit types'
    'status:Show staged changes'
    'amend:Amend the last commit'
    'uninstall:Remove cmt-managed hooks'
    'completions:Print shell completion scripts'
    'version:Print version'
    'help:Show help'
  )
  _describe 'command' cmds
}
compdef _cmt cmt
"#
    .to_string()
}

pub fn fish() -> String {
    r#"# cmt fish completions — add to config.fish:  cmt completions fish | source
complete -c cmt -f
complete -c cmt -n '__fish_use_subcommand' -a commit     -d 'Interactive commit builder'
complete -c cmt -n '__fish_use_subcommand' -a lint       -d 'Lint a commit message'
complete -c cmt -n '__fish_use_subcommand' -a init       -d 'Set up repo with hook and config'
complete -c cmt -n '__fish_use_subcommand' -a log        -d 'Pretty log of recent commits'
complete -c cmt -n '__fish_use_subcommand' -a types      -d 'List available commit types'
complete -c cmt -n '__fish_use_subcommand' -a status     -d 'Show staged changes'
complete -c cmt -n '__fish_use_subcommand' -a amend      -d 'Amend the last commit'
complete -c cmt -n '__fish_use_subcommand' -a uninstall  -d 'Remove cmt-managed hooks'
complete -c cmt -n '__fish_use_subcommand' -a completions -d 'Print shell completion scripts'
complete -c cmt -n '__fish_use_subcommand' -a version    -d 'Print version'
complete -c cmt -n '__fish_use_subcommand' -a help       -d 'Show help'
"#
    .to_string()
}

pub fn run(args: &[String]) {
    let shell = args.first().map(|s| s.as_str()).unwrap_or("bash");
    match shell {
        "bash" => print!("{}", bash()),
        "zsh" => print!("{}", zsh()),
        "fish" => print!("{}", fish()),
        other => {
            eprintln!("  ✖  Unknown shell: {other}  (use: bash, zsh, or fish)");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bash_lists_core_commands() {
        let s = bash();
        for c in [
            "commit",
            "lint",
            "init",
            "log",
            "types",
            "status",
            "amend",
            "uninstall",
            "completions",
            "version",
            "help",
        ] {
            assert!(s.contains(c), "missing {c}");
        }
    }
}
