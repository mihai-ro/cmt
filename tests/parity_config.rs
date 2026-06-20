use std::fs;
use std::path::Path;

#[test]
fn config_fixtures_resolve_canonically() {
    let dir = Path::new("tests/fixtures/config");
    let mut entries: Vec<_> = fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "json").unwrap_or(false))
        .collect();
    entries.sort();
    assert!(!entries.is_empty(), "no config fixtures found");
    for json in entries {
        let resolved = json.with_extension("resolved");
        let expected = fs::read_to_string(&resolved).unwrap();
        let cfg = cmt::config::load_from(&json);
        let got = cmt::config::dump_canonical(&cfg);
        assert_eq!(got, expected, "mismatch for {:?}", json);
    }
}
