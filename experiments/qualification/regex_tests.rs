#![cfg(feature = "compliant-regex")]

use arazzo_serde_qualification::iregexp_adapter::{compile, evaluate, take_backend_errors};
use serde_json::Value;

#[test]
fn normative_regex_cases() {
    let cases: Vec<Value> = serde_json::from_str(include_str!("../data/regex-cases.json")).unwrap();
    let mut failures = Vec::new();
    for case in cases {
        let (Some(pattern), Some(value)) = (case["pattern"].as_str(), case["value"].as_str())
        else {
            continue;
        };
        for (function, full) in [("match", true), ("search", false)] {
            let actual = evaluate(value, pattern, full);
            let expected = case[function].as_bool().unwrap();
            if actual != expected {
                failures.push(format!(
                    "{} / {function}: expected {expected}, actual {actual}",
                    case["name"]
                ));
            }
            let errors = take_backend_errors();
            if !errors.is_empty() {
                failures.push(format!(
                    "{} / {function}: backend errors {errors:?}",
                    case["name"]
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn valid_but_expensive_pattern_has_observable_backend_error() {
    assert!(compile("a{100000000}", true).is_err());
    assert!(!evaluate("a", "a{100000000}", true));
    assert!(!take_backend_errors().is_empty());
}

#[test]
fn independent_challenge_cases() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("../data/regex-challenge-cases.json")).unwrap();
    let mut failures = Vec::new();
    for case in cases {
        let actual = evaluate(
            case["value"].as_str().unwrap(),
            case["pattern"].as_str().unwrap(),
            case["full"].as_bool().unwrap(),
        );
        let errors = take_backend_errors();
        if actual != case["expected"].as_bool().unwrap() || !errors.is_empty() {
            failures.push(format!(
                "{} actual={actual}, expected={}, backend_errors={errors:?}",
                case["name"], case["expected"]
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
