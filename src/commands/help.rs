use crate::ui;

pub fn print_version() {
    println!("cmt version {}", env!("CARGO_PKG_VERSION"));
}

pub fn print_help() {
    let p = ui::palette();
    let v = env!("CARGO_PKG_VERSION");
    print!(
        "\n{bold}{cyan}cmt{reset} v{v} — Conventional Commits CLI\n\n{bold}USAGE{reset}\n  cmt <command> [options]\n\n{bold}COMMANDS{reset}\n  {cyan}init{reset} [--husky] [--lint]       Create .cmt.json + install prepare-commit-msg hook\n                        --husky  write to .husky/  (husky v9, committable)\n                        --lint   also install commit-msg linting hook\n                        default  write to .git/hooks/  (local)\n\n  {cyan}commit{reset} [--dry-run]         Interactive commit builder\n                        --dry-run  show assembled message, do not commit\n\n  {cyan}amend{reset}                    Amend the last commit (rebuild or raw edit)\n\n  {cyan}status{reset}                   Show staged changes\n\n  {cyan}lint{reset} [file]              Lint a commit message file or stdin\n\n  {cyan}log{reset} [n] [--type=<t>]     Pretty log — last n commits  (default: 20)\n      [--author=<a>]     filter by type or author substring\n\n  {cyan}types{reset}                    List available types\n\n  {cyan}completions{reset} [bash|zsh|fish]  Print shell completion script\n\n  {cyan}uninstall{reset}                Remove cmt-managed hooks\n\n{bold}EXAMPLES{reset}\n  cmt init                       # set up repo — done in one step\n  cmt commit                     # guided commit\n  cmt commit --dry-run           # preview message without committing\n  cmt amend                      # fix last commit\n  cmt log 10 --type=feat         # last 10 feat commits\n  echo 'fix: typo' | cmt lint\n\n{bold}SHELL COMPLETIONS{reset}\n  bash   Add to ~/.bashrc:    eval \"$(cmt completions bash)\"\n  zsh    Add to ~/.zshrc:     eval \"$(cmt completions zsh)\"\n  fish   Add to config.fish:  cmt completions fish | source\n\n{bold}CONFIG{reset}  .cmt.json  (JSON Schema -> intellisense in VS Code / JetBrains)\n\n",
        bold = p.bold,
        cyan = p.accent,
        reset = p.reset,
        v = v
    );
}
