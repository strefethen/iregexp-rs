use std::fmt::Write;

use pest::{
    error::{ErrorVariant, InputLocation},
    iterators::Pair,
    Parser,
};
use regex::RegexBuilder;

use crate::{Error, MatchMode};

include!(concat!(env!("OUT_DIR"), "/parser.rs"));

const MAX_PATTERN_BYTES: usize = 64 * 1024;
const MAX_OPENING_PARENTHESES: usize = 128;
const MAX_REPEAT_COUNT: usize = 10_000;
const MAX_BACKEND_NESTING: u32 = 256;
const MAX_COMPILED_BYTES: usize = 1024 * 1024;

pub(crate) fn translate(pattern: &str, mode: MatchMode) -> Result<String, Error> {
    check_preparse_limits(pattern)?;
    let mut pairs = IRegexpParser::parse(Rule::whole, pattern).map_err(syntax_error)?;
    let root = pairs.next().ok_or_else(|| Error::Backend {
        message: "generated parser returned no root node".to_owned(),
    })?;
    let mapped = render(root)?;
    match mode {
        MatchMode::Search => Ok(mapped),
        MatchMode::Full => Ok(format!(r"\A(?:{mapped})\z")),
    }
}

pub(crate) fn compile_backend(pattern: &str) -> Result<regex::Regex, Error> {
    RegexBuilder::new(pattern)
        .size_limit(MAX_COMPILED_BYTES)
        .nest_limit(MAX_BACKEND_NESTING)
        .build()
        .map_err(|error| match error {
            regex::Error::CompiledTooBig(limit) => Error::ResourceLimit {
                resource: "compiled matcher bytes",
                limit,
            },
            regex::Error::Syntax(message) => classify_backend_syntax(pattern, message),
            other => Error::Backend {
                message: other.to_string(),
            },
        })
}

fn check_preparse_limits(pattern: &str) -> Result<(), Error> {
    if pattern.len() > MAX_PATTERN_BYTES {
        return Err(Error::ResourceLimit {
            resource: "pattern bytes",
            limit: MAX_PATTERN_BYTES,
        });
    }
    if pattern.bytes().filter(|byte| *byte == b'(').count() > MAX_OPENING_PARENTHESES {
        return Err(Error::ResourceLimit {
            resource: "opening parentheses",
            limit: MAX_OPENING_PARENTHESES,
        });
    }
    Ok(())
}

fn syntax_error(error: pest::error::Error<Rule>) -> Error {
    let offset = match error.location {
        InputLocation::Pos(position) => position,
        InputLocation::Span((start, _)) => start,
    };
    let message = error.to_string();
    match error.variant {
        ErrorVariant::ParsingError { .. } => Error::Syntax { message, offset },
        ErrorVariant::CustomError { .. } => Error::Backend { message },
    }
}

fn classify_backend_syntax(pattern: &str, message: String) -> Error {
    let mut parser = regex_syntax::ast::parse::ParserBuilder::new();
    parser.nest_limit(MAX_BACKEND_NESTING);
    match parser.build().parse(pattern) {
        Err(error) => match error.kind() {
            regex_syntax::ast::ErrorKind::NestLimitExceeded(limit) => Error::ResourceLimit {
                resource: "backend syntax nesting",
                limit: *limit as usize,
            },
            _ => Error::Backend { message },
        },
        Ok(_) => Error::Backend { message },
    }
}

fn render(pair: Pair<'_, Rule>) -> Result<String, Error> {
    let rule = pair.as_rule();
    let source = pair.as_str();
    match rule {
        Rule::NormalChar => Ok(source.chars().map(literal).collect()),
        Rule::SingleCharEsc => Ok(literal(decode_escape(source).ok_or_else(|| {
            Error::Backend {
                message: "generated parser produced an invalid single-character escape".to_owned(),
            }
        })?)),
        Rule::CCchar if !source.starts_with('\\') => Ok(source.chars().map(literal).collect()),
        Rule::catEsc | Rule::complEsc => Ok(source.to_owned()),
        Rule::charClass if source == "." => Ok(r"[^\n\r]".to_owned()),
        Rule::charClassExpr => render_class(pair),
        Rule::range_quantifier => {
            validate_quantifier(&pair)?;
            render_children(pair)
        }
        _ => render_children(pair),
    }
}

fn render_class(pair: Pair<'_, Rule>) -> Result<String, Error> {
    let source = pair.as_str();
    let offset = pair.as_span().start();
    if source == "[^]" {
        return Err(Error::Syntax {
            message: "RFC 9485 explicitly prohibits the character class [^]".to_owned(),
            offset,
        });
    }

    for child in pair.clone().into_inner() {
        if child.as_rule() == Rule::CCE1 {
            validate_class_range(&child)?;
        }
    }

    let start = pair.as_span().start();
    let end = pair.as_span().end();
    let children: Vec<_> = pair.into_inner().collect();
    let negated = source.starts_with("[^");
    let mut output = if negated { "[^" } else { "[" }.to_owned();
    let prefix_end = children
        .first()
        .map_or(end - 1, |child| child.as_span().start());
    let prefix_start = start + if negated { 2 } else { 1 };
    for character in source[prefix_start - start..prefix_end - start].chars() {
        output.push_str(&literal(character));
    }
    let suffix_start = children
        .last()
        .map_or(prefix_end, |child| child.as_span().end());
    for child in children {
        output.push_str(&render(child)?);
    }
    for character in source[suffix_start - start..source.len() - 1].chars() {
        output.push_str(&literal(character));
    }
    output.push(']');
    Ok(output)
}

fn validate_class_range(pair: &Pair<'_, Rule>) -> Result<(), Error> {
    let endpoints: Vec<_> = pair
        .clone()
        .into_inner()
        .filter(|child| child.as_rule() == Rule::CCchar)
        .collect();
    if endpoints.len() != 2 {
        return Ok(());
    }
    let start = decode_class_character(&endpoints[0]).ok_or_else(|| Error::Backend {
        message: "generated parser produced an invalid range start".to_owned(),
    })?;
    let end = decode_class_character(&endpoints[1]).ok_or_else(|| Error::Backend {
        message: "generated parser produced an invalid range end".to_owned(),
    })?;
    if start > end {
        return Err(Error::Semantic {
            message: "character class range start must not exceed its end".to_owned(),
            offset: pair.as_span().start(),
        });
    }
    Ok(())
}

fn validate_quantifier(pair: &Pair<'_, Rule>) -> Result<(), Error> {
    let counts: Vec<_> = pair
        .clone()
        .into_inner()
        .filter(|child| child.as_rule() == Rule::QuantExact)
        .map(|child| parse_count(&child))
        .collect::<Result<_, _>>()?;
    if counts.len() == 2 && counts[0] > counts[1] {
        return Err(Error::Semantic {
            message: "range quantifier start must not exceed its end".to_owned(),
            offset: pair.as_span().start(),
        });
    }
    Ok(())
}

fn parse_count(pair: &Pair<'_, Rule>) -> Result<usize, Error> {
    let digits = pair.as_str().trim_start_matches('0');
    let count = if digits.is_empty() {
        0
    } else if digits.len() > MAX_REPEAT_COUNT.to_string().len() {
        return Err(Error::ResourceLimit {
            resource: "range quantifier count",
            limit: MAX_REPEAT_COUNT,
        });
    } else {
        digits.parse::<usize>().map_err(|error| Error::Backend {
            message: format!("generated numeric token could not be decoded: {error}"),
        })?
    };
    if count > MAX_REPEAT_COUNT {
        return Err(Error::ResourceLimit {
            resource: "range quantifier count",
            limit: MAX_REPEAT_COUNT,
        });
    }
    Ok(count)
}

fn render_children(pair: Pair<'_, Rule>) -> Result<String, Error> {
    let source = pair.as_str();
    let start = pair.as_span().start();
    let mut cursor = 0;
    let mut output = String::new();
    for child in pair.into_inner() {
        let child_start = child.as_span().start() - start;
        let child_end = child.as_span().end() - start;
        output.push_str(&source[cursor..child_start]);
        output.push_str(&render(child)?);
        cursor = child_end;
    }
    output.push_str(&source[cursor..]);
    Ok(output)
}

fn decode_class_character(pair: &Pair<'_, Rule>) -> Option<char> {
    let source = pair.as_str();
    if source.starts_with('\\') {
        decode_escape(source)
    } else {
        source.chars().next()
    }
}

fn decode_escape(source: &str) -> Option<char> {
    let mut characters = source.chars();
    if characters.next()? != '\\' {
        return None;
    }
    match characters.next()? {
        'n' => Some('\n'),
        'r' => Some('\r'),
        't' => Some('\t'),
        other => Some(other),
    }
}

fn literal(character: char) -> String {
    let mut escaped = String::new();
    write!(escaped, r"\x{{{:x}}}", u32::from(character)).expect("writing to a String cannot fail");
    escaped
}

#[cfg(test)]
mod tests {
    use super::{
        compile_backend, translate, MAX_BACKEND_NESTING, MAX_OPENING_PARENTHESES,
        MAX_PATTERN_BYTES, MAX_REPEAT_COUNT,
    };
    use crate::{Error, MatchMode};

    #[test]
    fn translates_metacharacters_as_literals() {
        assert_eq!(translate("^$", MatchMode::Search).unwrap(), r"\x{5e}\x{24}");
        assert_eq!(translate(".", MatchMode::Search).unwrap(), r"[^\n\r]");
        assert_eq!(
            translate("[a&&b]", MatchMode::Search).unwrap(),
            r"[\x{61}\x{26}\x{26}\x{62}]"
        );
    }

    #[test]
    fn preserves_zero_padded_counts() {
        let pattern = format!("a{{{}{}}}", "0".repeat(100), MAX_REPEAT_COUNT);
        assert!(translate(&pattern, MatchMode::Search).is_ok());
    }

    #[test]
    fn rejects_limits_before_recursive_parsing() {
        let too_large = "a".repeat(MAX_PATTERN_BYTES + 1);
        assert!(matches!(
            translate(&too_large, MatchMode::Search),
            Err(Error::ResourceLimit { .. })
        ));
        let too_many_groups = "(".repeat(MAX_OPENING_PARENTHESES + 1);
        assert!(matches!(
            translate(&too_many_groups, MatchMode::Search),
            Err(Error::ResourceLimit { .. })
        ));
        let too_large_repeat = format!("a{{{}}}", MAX_REPEAT_COUNT + 1);
        assert!(matches!(
            translate(&too_large_repeat, MatchMode::Search),
            Err(Error::ResourceLimit { .. })
        ));
    }

    #[test]
    fn backend_failures_keep_resource_and_backend_errors_distinct() {
        assert!(matches!(
            compile_backend("(?=a)"),
            Err(Error::Backend { .. })
        ));

        let depth = MAX_BACKEND_NESTING as usize + 1;
        let nested = format!("{}a{}", "(".repeat(depth), ")".repeat(depth));
        assert!(matches!(
            compile_backend(&nested),
            Err(Error::ResourceLimit { .. })
        ));
    }

    #[test]
    fn custom_generated_parser_failures_are_backend_errors() {
        let position = pest::Position::from_start("pattern");
        let parser_error = pest::error::Error::new_from_pos(
            pest::error::ErrorVariant::CustomError {
                message: "external parser limit".to_owned(),
            },
            position,
        );
        assert!(matches!(
            super::syntax_error(parser_error),
            Error::Backend { .. }
        ));
    }
}
