use iregexp::{IRegexp, MatchMode};
use serde_json::Value;

fn rows(source: &str) -> Vec<Value> {
    serde_json::from_str(source).expect("checked-in fixture JSON")
}

#[test]
fn all_main_fixtures() {
    let cases = rows(include_str!("fixtures/regex-cases.json"));
    assert_eq!(cases.len(), 192);
    let mut excluded = Vec::new();
    let mut valid_count = 0;
    let mut invalid_count = 0;
    for case in &cases {
        let name = case["name"].as_str().unwrap();
        let (Some(pattern), Some(value)) = (case["pattern"].as_str(), case["value"].as_str())
        else {
            // These four rows specify JSONPath argument typing. The library's
            // &str API cannot represent them; retain and account for each row.
            excluded.push(name);
            if let Some(pattern) = case["pattern"].as_str() {
                assert!(IRegexp::compile(pattern, MatchMode::Full).is_ok());
            }
            continue;
        };
        let valid = case["valid"].as_bool().unwrap();
        if valid {
            valid_count += 1;
        } else {
            invalid_count += 1;
        }
        for (mode, expected_key) in [(MatchMode::Full, "match"), (MatchMode::Search, "search")] {
            let compiled = IRegexp::compile(pattern, mode);
            if valid {
                let compiled = compiled.unwrap_or_else(|error| panic!("{name}: {error:?}"));
                assert_eq!(
                    compiled.is_match(value),
                    case[expected_key].as_bool().unwrap(),
                    "{name}: {mode:?}"
                );
            } else {
                // Never accept resource/backend failures as invalidity proof.
                let error = compiled.expect_err(name);
                assert!(
                    matches!(
                        error,
                        iregexp::Error::Syntax { .. } | iregexp::Error::Semantic { .. }
                    ),
                    "{name}: {error:?}"
                );
            }
        }
    }
    assert_eq!((valid_count, invalid_count), (146, 42));
    assert_eq!(
        excluded,
        [
            "type_nonstring_value",
            "type_nonstring_pattern",
            "type_null_value",
            "type_null_pattern"
        ]
    );
}

#[test]
fn all_independent_challenge_fixtures() {
    let cases = rows(include_str!("fixtures/regex-challenge-cases.json"));
    assert_eq!(cases.len(), 63);
    // These patterns violate RFC 9485 section 3; the fixture's false outcome
    // alone does not establish invalidity. All other rows must compile.
    let invalid = [
        "dollar escape invalid",
        "formfeed escape invalid",
        "unicode codepoint escape invalid",
        "surrogate category name invalid",
        "empty class invalid",
        "prohibited caret-only class",
        "category range endpoint invalid",
        "two category range invalid",
        "nested class syntax invalid",
        "ascii named class syntax invalid",
    ];
    let mut invalid_count = 0;
    for case in cases {
        let name = case["name"].as_str().unwrap();
        let mode = if case["full"].as_bool().unwrap() {
            MatchMode::Full
        } else {
            MatchMode::Search
        };
        let compiled = IRegexp::compile(case["pattern"].as_str().unwrap(), mode);
        if invalid.contains(&name) {
            invalid_count += 1;
            assert!(!case["expected"].as_bool().unwrap());
            assert!(
                matches!(compiled, Err(iregexp::Error::Syntax { .. })),
                "{name}: {compiled:?}"
            );
        } else {
            let compiled = compiled.unwrap_or_else(|error| panic!("{name}: {error:?}"));
            assert_eq!(
                compiled.is_match(case["value"].as_str().unwrap()),
                case["expected"].as_bool().unwrap(),
                "{name}"
            );
        }
    }
    assert_eq!(invalid_count, invalid.len());
}

#[test]
fn all_categories_and_complements_in_every_class_form() {
    let cases = rows(include_str!("fixtures/regex-cases.json"));
    let mut count = 0;
    for case in &cases {
        let name = case["name"].as_str().unwrap();
        let Some(category) = name.strip_prefix("category_") else {
            continue;
        };
        if category.len() > 2 {
            continue;
        }
        count += 1;
        let member = case["value"].as_str().unwrap();
        let outsider = if category.starts_with('L') { "1" } else { "A" };
        for (pattern, positive) in [
            (format!("\\p{{{category}}}"), true),
            (format!("\\P{{{category}}}"), false),
            (format!("[\\p{{{category}}}]"), true),
            (format!("[\\P{{{category}}}]"), false),
            (format!("[^\\p{{{category}}}]"), false),
            (format!("[^\\P{{{category}}}]"), true),
        ] {
            for mode in [MatchMode::Full, MatchMode::Search] {
                let regex = IRegexp::compile(&pattern, mode).unwrap();
                assert_eq!(regex.is_match(member), positive, "{pattern}: member");
                assert_eq!(regex.is_match(outsider), !positive, "{pattern}: outsider");
            }
        }
    }
    assert_eq!(count, 36);
}
