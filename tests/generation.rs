#[test]
fn generation_retains_every_normative_rule_and_license() {
    let source = include_str!("../grammar/rfc9485.abnf");
    let generated = include_str!(concat!(env!("OUT_DIR"), "/generated.pest"));
    let parser = include_str!(concat!(env!("OUT_DIR"), "/parser.rs"));
    let source_rules: Vec<_> = source
        .lines()
        .filter_map(|line| line.split_once(" = "))
        .collect();
    assert_eq!(source_rules.len(), 25);
    for (name, _) in source_rules {
        let declaration = format!("{} = ", name.replace('-', "_"));
        assert!(
            generated.lines().any(|line| line.starts_with(&declaration)),
            "missing {name}"
        );
    }
    assert_eq!(
        generated
            .lines()
            .filter(|line| !line.starts_with("//") && line.contains(" = "))
            .count(),
        26
    );
    assert!(generated.contains("whole = { SOI ~ i_regexp ~ EOI }"));
    for line in include_str!("../grammar/LICENSE-RFC9485.txt").lines() {
        assert!(generated.contains(&format!("// {line}\n")));
        assert!(parser.contains(&format!("// {line}\n")));
    }
}
