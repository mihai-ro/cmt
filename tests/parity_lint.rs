use std::fs;
use std::path::Path;

#[test]
fn lint_fixtures_match() {
    // Fixtures were captured with NO_COLOR=1.
    // SAFETY: single-threaded test binary, no other threads read env.
    unsafe { std::env::set_var("NO_COLOR", "1") };
    let dir = Path::new("tests/fixtures/lint");
    let mut names: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            e.path()
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
        })
        .collect();
    names.sort();
    names.dedup();
    assert!(!names.is_empty());
    let cfg = cmt::types::Config::default();
    let p = cmt::ui::Palette::plain();
    for name in names {
        let msg = fs::read_to_string(dir.join(format!("{name}.msg"))).unwrap();
        let expected_out = fs::read_to_string(dir.join(format!("{name}.out")))
            .unwrap()
            .replace("\r\n", "\n");
        let expected_code: i32 = fs::read_to_string(dir.join(format!("{name}.code")))
            .unwrap()
            .trim()
            .parse()
            .unwrap();
        let result = cmt::lint::check(&msg, &cfg);
        let rendered = cmt::lint::render(&result, &p);
        assert_eq!(rendered, expected_out, "stdout mismatch for {name}");
        assert_eq!(
            result.exit_code(),
            expected_code,
            "exit code mismatch for {name}"
        );
    }
}
