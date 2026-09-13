use pest::Parser;
use serde_json::json;
include!(concat!(env!("OUT_DIR"), "/parser.rs"));
fn main() {
    let cases = [
        ("empty", "", true),
        ("multi-digit exact", "a{12}", true),
        ("multi-digit range", "a{12,34}", true),
        ("lowercase category escape", r"\p{Lu}", true),
        ("uppercase complement escape", r"\P{Lu}", true),
        ("case-sensitive category", r"\p{lu}", false),
        ("astral scalar", "😀", true),
        ("highest scalar", "\u{10ffff}", true),
        ("private-use boundary", "\u{e000}", true),
        ("literal anchors", "^a$", true),
        ("case flag extension", "(?i)a", false),
        ("shorthand extension", r"\d", false),
        ("trailing junk", "a)", false),
        ("positive range", "[a-z]", true),
        ("trailing dash", "[a-]", true),
        ("leading dash", "[-a]", true),
        ("negated class", "[^a]", true),
        ("negated caret", "[^^]", true),
        ("explicit RFC exclusion", "[^]", false),
        ("semantic reversed range", "[z-a]", false),
        ("semantic reversed count", "a{3,2}", false),
        ("alternation", "(a|ab)c", true),
        ("class category escape", r"[\p{Lu}a-z]", true),
    ];
    let observations: Vec<_> = cases.into_iter().map(|(name, pattern, expected)| {
        let parsed = IRegexpParser::parse(Rule::whole, pattern);
        json!({"name":name,"pattern":pattern,"expected_full_iregexp_validity":expected,"generated_parser_accepts":parsed.is_ok(),"error":parsed.err().map(|e|e.to_string())})
    }).collect();
    println!("{}", serde_json::to_string_pretty(&json!({"generator":"abnf_to_pest 0.6.0","pest":"2.9.1","rules":include_str!(concat!(env!("OUT_DIR"), "/generated.pest")),"observations":observations})).unwrap());
}
