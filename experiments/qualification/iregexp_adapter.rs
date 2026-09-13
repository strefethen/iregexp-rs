//! Experimental RFC 9485 mapping over an external Pest parse tree.
//!
//! This is qualification code, not an approved production component. The
//! upstream checker has one semantic grammar repair and three visible rules.
//! JSONPath parsing, typing, and evaluation remain wholly upstream-owned.

use iregexp::{IRegexp, Rule};
use pest::{iterators::Pair, Parser};
use regex::Regex;
use serde_json_path::functions::{LogicalType, ValueType};
use std::cell::RefCell;
use std::fmt::Write;

#[derive(Debug)]
pub enum PatternError {
    InvalidIRegexp,
    MatcherRejected(String),
}

thread_local! {
    // A LogicalType callback cannot return a resource error. Expose such errors
    // to the harness rather than claim every false result is semantic proof.
    static BACKEND_ERRORS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

pub fn take_backend_errors() -> Vec<String> {
    BACKEND_ERRORS.with(|errors| std::mem::take(&mut *errors.borrow_mut()))
}

pub fn ensure_linked() {}

fn literal(character: char) -> String {
    format!("\\x{{{:x}}}", u32::from(character))
}

fn render(pair: Pair<'_, Rule>) -> String {
    let rule = pair.as_rule();
    let source = pair.as_str();
    match rule {
        Rule::normal_char => source.chars().map(literal).collect(),
        Rule::single_char_esc => {
            // The external grammar has already recognized exactly two chars.
            let character = source.chars().nth(1).unwrap_or_default();
            literal(match character {
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                other => other,
            })
        }
        Rule::c_char if !source.starts_with('\\') => source.chars().map(literal).collect(),
        Rule::cat_esc | Rule::compl_esc => source.to_owned(),
        Rule::char_class if source == "." => "[^\\n\\r]".to_owned(),
        Rule::char_class_expr => {
            let start = pair.as_span().start();
            let end = pair.as_span().end();
            let children: Vec<_> = pair.into_inner().collect();
            let negated = source.starts_with("[^");
            let mut output = if negated { "[^" } else { "[" }.to_owned();
            // Only leading/trailing hyphens are outside cce1 pairs; ranges
            // remain owned by the external cce1 grammar and generic renderer.
            let prefix_end = children.first().map_or(end - 1, |p| p.as_span().start());
            let prefix_start = start + if negated { 2 } else { 1 };
            for character in source[prefix_start - start..prefix_end - start].chars() {
                output.push_str(&literal(character));
            }
            let suffix_start = children.last().map_or(prefix_end, |p| p.as_span().end());
            for child in children {
                output.push_str(&render(child));
            }
            for character in source[suffix_start - start..source.len() - 1].chars() {
                output.push_str(&literal(character));
            }
            output.push(']');
            output
        }
        _ => {
            let start = pair.as_span().start();
            let mut cursor = 0;
            let mut output = String::new();
            for child in pair.into_inner() {
                let child_start = child.as_span().start() - start;
                let child_end = child.as_span().end() - start;
                output.push_str(&source[cursor..child_start]);
                output.push_str(&render(child));
                cursor = child_end;
            }
            output.push_str(&source[cursor..]);
            output
        }
    }
}

pub fn translated_pattern(pattern: &str, full: bool) -> Result<String, PatternError> {
    let mut tree =
        IRegexp::parse(Rule::pattern, pattern).map_err(|_| PatternError::InvalidIRegexp)?;
    let root = tree.next().ok_or(PatternError::InvalidIRegexp)?;
    let mapped = render(root);
    if full {
        let mut result = String::with_capacity(mapped.len() + 9);
        // Absolute boundaries also make alternation apply to the whole string.
        let _ = write!(result, "\\A(?:{mapped})\\z");
        Ok(result)
    } else {
        Ok(mapped)
    }
}

pub fn compile(pattern: &str, full: bool) -> Result<Regex, PatternError> {
    let mapped = translated_pattern(pattern, full)?;
    Regex::new(&mapped).map_err(|error| {
        // RFC 9485 section 4 imports XSD 1.0 range constraints. The checker
        // recognizes their grammar but does not reject reversed bounds. Ask
        // the established backend parser for structured error kinds; do not
        // inspect diagnostic strings or implement another range parser.
        if matches!(&error, regex::Error::Syntax(_)) {
            if let Err(syntax) = regex_syntax::ast::parse::Parser::new().parse(&mapped) {
                use regex_syntax::ast::ErrorKind;
                if matches!(
                    syntax.kind(),
                    ErrorKind::ClassRangeInvalid | ErrorKind::RepetitionCountInvalid
                ) {
                    return PatternError::InvalidIRegexp;
                }
            }
        }
        PatternError::MatcherRejected(error.to_string())
    })
}

pub fn evaluate(value: &str, pattern: &str, full: bool) -> bool {
    match compile(pattern, full) {
        Ok(compiled) => compiled.is_match(value),
        Err(PatternError::InvalidIRegexp) => false,
        Err(PatternError::MatcherRejected(error)) => {
            BACKEND_ERRORS.with(|errors| errors.borrow_mut().push(error));
            false
        }
    }
}

fn callback(value: ValueType<'_>, pattern: ValueType<'_>, full: bool) -> LogicalType {
    let Some(value) = value.as_value().and_then(serde_json::Value::as_str) else {
        return false.into();
    };
    let Some(pattern) = pattern.as_value().and_then(serde_json::Value::as_str) else {
        return false.into();
    };
    evaluate(value, pattern, full).into()
}

#[serde_json_path::function(name = "match")]
fn standard_match(value: ValueType<'_>, pattern: ValueType<'_>) -> LogicalType {
    callback(value, pattern, true)
}

#[serde_json_path::function(name = "search")]
fn standard_search(value: ValueType<'_>, pattern: ValueType<'_>) -> LogicalType {
    callback(value, pattern, false)
}
