use cmt::commit::{assemble, Draft};
use std::fs;
use std::path::Path;

fn parse_in(path: &Path) -> Draft {
    let mut d = Draft {
        type_: String::new(),
        scope: String::new(),
        desc: String::new(),
        body: String::new(),
        breaking: false,
        breaking_desc: String::new(),
        footer: String::new(),
    };
    for line in fs::read_to_string(path).unwrap().lines() {
        let (k, v) = line.split_once('=').unwrap_or((line, ""));
        match k {
            "type" => d.type_ = v.into(),
            "scope" => d.scope = v.into(),
            "desc" => d.desc = v.into(),
            "breaking" => d.breaking = v == "1",
            "breaking_desc" => d.breaking_desc = v.into(),
            "footer" => d.footer = v.into(),
            "body" => d.body = v.replace("\\n", "\n"),
            _ => {}
        }
    }
    d
}

#[test]
fn assemble_fixtures_match() {
    let dir = Path::new("tests/fixtures/assemble");
    let mut ins: Vec<_> = fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "in").unwrap_or(false))
        .collect();
    ins.sort();
    assert!(!ins.is_empty());
    for input in ins {
        let d = parse_in(&input);
        let expected = fs::read_to_string(input.with_extension("out")).unwrap();
        assert_eq!(assemble(&d), expected, "mismatch for {:?}", input);
    }
}
