use iregexp::{IRegexp, MatchMode};

fn matches(pattern: &str, text: &str) -> bool {
    IRegexp::compile(pattern, MatchMode::Full)
        .unwrap()
        .is_match(text)
}

#[test]
fn every_permitted_escape_inside_and_outside_classes() {
    for character in "()*+-.?[\\]^{|}".chars() {
        let escaped = format!("\\{character}");
        assert!(matches(&escaped, &character.to_string()), "{escaped}");
        assert!(
            matches(&format!("[{escaped}]"), &character.to_string()),
            "{escaped}"
        );
    }
    for (escaped, character) in [(r"\n", '\n'), (r"\r", '\r'), (r"\t", '\t')] {
        assert!(matches(escaped, &character.to_string()));
        assert!(matches(&format!("[{escaped}]"), &character.to_string()));
    }
}

#[test]
fn scalar_boundaries_and_whitespace_are_literal() {
    for character in [
        '\0',
        '\u{27}',
        '\u{7e}',
        '\u{d7ff}',
        '\u{e000}',
        '\u{ffff}',
        '\u{10000}',
        '\u{10ffff}',
        ' ',
        '\t',
        '\n',
        '\r',
    ] {
        let text = character.to_string();
        assert!(matches(&text, &text));
        assert!(matches(&format!("[{text}]"), &text));
        assert!(!matches(&format!("[^{text}]"), &text));
    }
    assert!(matches(" a # b ", " a # b "));
    assert!(!matches(" a # b ", "a#b"));
}

#[test]
fn whole_input_and_search_are_distinct() {
    for (pattern, text) in [
        ("a|ab", "ab"),
        ("a?|abc", "abc"),
        ("|ab", "ab"),
        ("(a|ab){2}", "abab"),
    ] {
        assert!(matches(pattern, text), "{pattern}");
        assert!(!matches(pattern, &format!("x{text}")), "{pattern}");
        assert!(IRegexp::compile(pattern, MatchMode::Search)
            .unwrap()
            .is_match(&format!("x{text}y")));
    }
    assert!(matches("", ""));
    assert!(!matches("", "\n"));
    assert!(IRegexp::compile("", MatchMode::Search)
        .unwrap()
        .is_match("anything"));
    assert!(!matches("a", "a\n"));
    assert!(!matches("a", "a\r\n"));
    for text in ["\u{85}", "\u{2028}", "\u{2029}", "😀"] {
        assert!(matches(".", text));
    }
    for text in ["\r", "\n", "\r\n"] {
        assert!(!matches(".", text));
    }
}

#[test]
fn generated_grammar_ordering_and_literal_class_operators() {
    for (pattern, member, outsider) in [
        ("[^^]", "x", "^"),
        ("[^-x]", "y", "-"),
        ("[a-]", "-", "b"),
        ("[a-b-]", "b", "c"),
        ("[--]", "-", "z"),
        ("[a&&b]", "&", "c"),
        ("[a~~b]", "~", "c"),
        ("[a||b]", "|", "c"),
    ] {
        assert!(matches(pattern, member), "{pattern}");
        assert!(!matches(pattern, outsider), "{pattern}");
    }
    for pattern in [
        "[a-b-c]",
        "[--z]",
        "[^]",
        "[]",
        "[a",
        "a)",
        "a{1}x)",
        "(?i)a",
        "a*?",
        "a++",
        r"\d",
        r"\p{l}",
        r"\p{Cs}",
        r"\p{IsBasicLatin}",
    ] {
        assert!(
            matches!(
                IRegexp::compile(pattern, MatchMode::Full),
                Err(iregexp::Error::Syntax { .. })
            ),
            "{pattern}"
        );
    }
}

#[test]
fn repetitions_and_semantic_rejections() {
    assert!(matches("a{12}", &"a".repeat(12)));
    assert!(matches("a{10,12}", &"a".repeat(11)));
    assert!(matches("a{12,}", &"a".repeat(15)));
    assert!(matches("a{00012}", &"a".repeat(12)));
    assert!(matches("a{0,0}", ""));
    for pattern in ["[z-a]", "[ω-α]", r"[\r-\n]", "a{3,2}", "a{0010,0002}"] {
        assert!(
            matches!(
                IRegexp::compile(pattern, MatchMode::Full),
                Err(iregexp::Error::Semantic { .. })
            ),
            "{pattern}"
        );
    }
}
