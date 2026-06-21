use cmt::commands::{amend, completions, help, init, log, status, types_cmd, uninstall};
use cmt::{commit, config, lint, ui};
use std::io::Read;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = args
        .first()
        .cloned()
        .unwrap_or_else(|| "commit".to_string());
    let rest: Vec<String> = args.iter().skip(1).cloned().collect();

    match cmd.as_str() {
        "commit" | "c" => {
            let write_only = rest.iter().any(|a| a == "--write-only");
            let dry_run = rest.iter().any(|a| a == "--dry-run");
            commit::run(write_only, dry_run);
        }
        "amend" | "a" => amend::run(&rest),
        "lint" | "l" => run_lint(&rest),
        "init" => init::run(&rest),
        "log" => log::run(&rest),
        "status" | "s" => status::run(&rest),
        "types" | "t" => types_cmd::run(&rest),
        "completions" => completions::run(&rest),
        "uninstall" => uninstall::run(&rest),
        "version" | "-v" | "--version" => help::print_version(),
        _ => help::print_help(),
    }
}

fn run_lint(args: &[String]) {
    let p = ui::palette();
    let msg = if let Some(file) = args.first() {
        match std::fs::read_to_string(file) {
            Ok(s) => s,
            Err(_) => {
                eprintln!("  {}✖{}  File not found: {}", p.red, p.reset, file);
                std::process::exit(1);
            }
        }
    } else {
        let mut s = String::new();
        let _ = std::io::stdin().read_to_string(&mut s);
        s
    };
    let cfg = config::load();
    let result = lint::check(&msg, &cfg);
    print!("{}", lint::render(&result, &p));
    std::process::exit(result.exit_code());
}
