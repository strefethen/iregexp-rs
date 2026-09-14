# iregexp-rs

A reusable checking and matching implementation of
[RFC 9485 I-Regexp](https://www.rfc-editor.org/rfc/rfc9485). Its parser is
generated from the RFC's ABNF, and its public API accepts strings and returns
structured errors.

```rust
use iregexp::{IRegexp, MatchMode};

let full = IRegexp::compile("a|ab", MatchMode::Full).unwrap();
assert!(full.is_match("ab"));
assert!(!full.is_match("xaby"));

let search = IRegexp::compile("a|ab", MatchMode::Search).unwrap();
assert!(search.is_match("xaby"));
```

Compile once and reuse the matcher. `Full` requires the entire string to match;
`Search` accepts any matching substring. Empty patterns match only an empty
string in `Full` mode and every string in `Search` mode. Alternation participates
in whole-string matching, even when its first alternative is shorter.

## Semantics

- All 36 RFC general-category names and their complements are supported on
  Unicode scalar values. Rust `str` excludes surrogate code points; `Cs` is not
  an allowed RFC category. Category data comes from `regex-syntax`; this
  release is qualified with regex-syntax 0.8.11 (Unicode 16.0.0). Unicode
  normalization is not performed.
- `.` matches one scalar value except CR and LF. Other line separators match.
- `^` and `$` are literals. Escaping `$`, `\d`, flags, lookaround, backreferences,
  lazy quantifiers, class subtraction and Unicode block names are invalid.
- Class members such as `&&`, `~~` and `||` are literals, and ranges use code-point
  order. Reversed ranges and reversed repetition bounds are semantic errors.
- Whitespace and NUL are significant. Parsing consumes the entire pattern.

The library implements the RFC's checking subset and the XSD semantics required
by RFC 9485 section 4. `Search` provides an additional substring matching mode.
Pattern errors are never converted into a false match.

## Errors and limits

`Error` distinguishes `Syntax`, `Semantic`, `ResourceLimit`, and `Backend`.
`Backend` reports an unexpected parser/matcher rejection that is not evidence
of invalid I-Regexp. Error messages are diagnostic text, not a stable parsing
interface. Match results are boolean after compilation succeeds.

Compilation has fixed conservative limits:

| Resource | Limit |
| --- | ---: |
| UTF-8 pattern bytes | 65,536 |
| Opening parenthesis bytes in the input | 128 |
| Numeric repetition bound | 10,000 |
| Backend syntax nesting | 256 |
| Compiled matcher size budget | 1 MiB |

The parenthesis guard counts raw `(` bytes, including escaped and class literals.
This deliberately conservative resource policy runs before recursive parsing;
it is not a second syntax checker. Limits can therefore precede invalidity
diagnostics. No process-global Pest configuration is changed. Leading zeroes
do not increase a repetition's numeric value.

These limits bound accepted compilation work; they are not an allocation quota
or a wall-clock deadline. Matching uses Rust regex's finite-automata engine,
whose worst-case search time scales with compiled-pattern size and text length.
The input text has no library-imposed length cap. Callers own request-level
input limits and concurrency budgets.

## Installation

The crate has not yet been published to crates.io. It can be used directly from
the source repository:

```toml
[dependencies]
iregexp = { package = "iregexp-rs", git = "https://github.com/strefethen/iregexp-rs" }
```

## Build

Rust 1.85 is the declared minimum supported version. The package is named
`iregexp-rs` and exports the library as `iregexp`.

`build.rs` converts the attributed [RFC grammar](grammar/rfc9485.abnf) using
`abnf_to_pest`, then Pest generates the private Rust parser. Derived files and
their RFC notice remain in Cargo `OUT_DIR`; no generated parser is checked in.

Run the complete local verification suite with:

```sh
./scripts/verify.sh
```

The script checks formatting, Clippy, tests, rustdoc, the declared MSRV, and a
local packaged build.

## License

The original project source is licensed under the [MIT License](LICENSE). The
RFC grammar is an IETF code component distributed under the Revised BSD License;
its required attribution and terms are retained in
[grammar/LICENSE-RFC9485.txt](grammar/LICENSE-RFC9485.txt). See
[THIRD-PARTY.md](THIRD-PARTY.md) for full source attribution.
