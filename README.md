# iregexp-rs

A checking implementation of [RFC 9485 I-Regexp](https://www.rfc-editor.org/rfc/rfc9485),
with a parser generated from the RFC's ABNF and a private adapter to Rust `regex`.
The public API accepts strings and returns structured errors; it has no JSONPath
or Arazzo dependencies.

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
  an allowed RFC category. The pinned matcher uses Unicode 16.0.0 data.
  Unicode normalization is not performed.
- `.` matches one scalar value except CR and LF. Other line separators match.
- `^` and `$` are literals. Escaping `$`, `\d`, flags, lookaround, backreferences,
  lazy quantifiers, class subtraction and Unicode block names are invalid.
- Class members such as `&&`, `~~` and `||` are literals, and ranges use code-point
  order. Reversed ranges and reversed repetition bounds are semantic errors.
- Whitespace and NUL are significant. Parsing consumes the entire pattern.

The library implements the RFC's checking subset. It uses XSD semantics as
required by RFC 9485 section 4, with the additional substring mode useful to
consumers such as RFC 9535. Pattern errors are never converted into a false match.

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

## Build and qualification

Rust 1.85 is the declared minimum supported version. The package is named
`iregexp-rs` and exports library `iregexp`. For local use:

```toml
[dependencies]
iregexp = { package = "iregexp-rs", path = "../iregexp-rs" }
```

`build.rs` converts the unchanged [RFC grammar](grammar/rfc9485.abnf) using
`abnf_to_pest`, then Pest generates the private Rust parser. Derived files and
their RFC notice remain in Cargo `OUT_DIR`; no generated parser is checked in.
The private adapter consumes the generated tree and safely encodes literals.

From a clean checkout, with Rust 1.85 installed, run:

```sh
./scripts/verify.sh
```

This verifies imported checksums, formatting, all-target/all-feature checking
and strict Clippy, debug/release tests, rustdoc, MSRV tests, local packaging,
the dependency tree, and handwritten line counts. Cargo's packaging step builds
the unpacked crate locally; it does not publish anything.

Tests assert 188 string/string rows from the 192-row main corpus and account
explicitly for four JSONPath argument-type rows that the `&str` API cannot
represent. They also assert all 63 independent challenge rows, distinguishing
the ten syntax-invalid patterns from valid patterns whose match result is false.
Additional tests cover all category complements, syntax ordering, scalar and
escape boundaries, matching modes, resource boundaries and reuse after errors.

The [handoff](HANDOFF.md), [prior proof](experiments/codegen-proof/README.md),
and [provenance manifest](PROVENANCE.json) preserve the source of the approved
work. The proof is historical; the root library's asserting suite owns current
behavior. See [THIRD-PARTY.md](THIRD-PARTY.md) for source and license attribution.

This is a local repository and working crate name. No registry name is reserved,
no remote has been created, and nothing has been published. Publication and
Arazzo integration are separate work.
