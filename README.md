# iregexp-rs

[![CI](https://github.com/strefethen/iregexp-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/strefethen/iregexp-rs/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/strefethen/iregexp-rs)](https://github.com/strefethen/iregexp-rs/releases)
[![Rust](https://img.shields.io/badge/Rust-1.85+-000000?logo=rust)](https://www.rust-lang.org/)
[![License: MIT AND BSD-3-Clause](https://img.shields.io/badge/License-MIT%20AND%20BSD--3--Clause-yellow.svg)](#license)

**RFC 9485 I-Regexp checking and matching for Rust, with a parser generated from the RFC's own grammar.**

iregexp-rs implements [I-Regexp](https://www.rfc-editor.org/rfc/rfc9485), the
IETF's interoperable regular-expression flavor and the pattern language behind
the JSONPath `match()` and `search()` functions
([RFC 9535](https://www.rfc-editor.org/rfc/rfc9535)). It rejects patterns
outside the RFC with a structured error, and compiles valid patterns into
reusable matchers.

## Why?

Regular-expression dialects disagree. `\d`, `(?i)`, lookaround, backreferences
and `^`/`$` behave differently across engines, and some engines don't support
them at all. A pattern one library accepts can be rejected or reinterpreted by
another. RFC 9485 defines a small subset meant to behave the same everywhere.
General-purpose engines, Rust's `regex` crate included, accept far more than
that subset, and they treat `^` and `$` as anchors where I-Regexp treats them as
literal characters.

iregexp-rs closes that gap. Every pattern is checked against the RFC grammar and
its semantic restrictions first. The pattern is then translated for the
[`regex`](https://crates.io/crates/regex) crate's finite-automata engine using
I-Regexp semantics. A pattern that isn't I-Regexp is never silently treated as
a non-match.

## Quick Start

iregexp-rs is distributed from GitHub, not crates.io. Add it as a git dependency
pinned to a release tag:

```toml
[dependencies]
iregexp = { package = "iregexp-rs", git = "https://github.com/strefethen/iregexp-rs", tag = "v0.1.0" }
```

The package is named `iregexp-rs`, and the library is imported as `iregexp`:

```rust
use iregexp::{IRegexp, MatchMode};

fn main() -> Result<(), iregexp::Error> {
    let full = IRegexp::compile("a|ab", MatchMode::Full)?;
    assert!(full.is_match("ab"));
    assert!(!full.is_match("xaby"));

    let search = IRegexp::compile("a|ab", MatchMode::Search)?;
    assert!(search.is_match("xaby"));

    let name = IRegexp::compile(r"\p{Lu}\p{Ll}+", MatchMode::Full)?;
    assert!(name.is_match("Émile"));
    Ok(())
}
```

## Features

| Feature | What it does |
|---|---|
| **Generated parser** | `build.rs` converts the RFC's ABNF into a Pest grammar at build time. There is no hand-written lexer or grammar to drift from the spec |
| **Full and Search modes** | `Full` matches the entire string, like JSONPath `match()`. `Search` matches any substring, like `search()` |
| **Semantic checks** | Reversed character ranges and repetition bounds are rejected, although the ABNF alone admits them |
| **Unicode categories** | All 36 RFC general-category names, plus their `\P{..}` complements |
| **Structured errors** | `Syntax`, `Semantic`, `ResourceLimit` and `Backend` failures stay distinguishable. Syntax and semantic errors carry byte offsets |
| **Resource limits** | Fixed, documented bounds on pattern size, nesting, repetition and compiled matcher size |
| **No backtracking** | Matching uses Rust's `regex` finite-automata engine |
| **Safe and reusable** | `#![forbid(unsafe_code)]`. A compiled `IRegexp` is `Send + Sync`: compile it once and share it |

## Contents

- [Usage](#usage)
- [Semantics](#semantics)
- [Errors and Limits](#errors-and-limits)
- [How It Works](#how-it-works)
- [Testing and Provenance](#testing-and-provenance)
- [Repository Layout](#repository-layout)
- [Building from Source](#building-from-source)
- [Project Status](#project-status)
- [Contributing](#contributing)
- [Acknowledgments](#acknowledgments)
- [License](#license)

## Usage

### Matching modes

Compile once and reuse the matcher. `Full` requires the entire string to match;
`Search` accepts any matching substring. An empty pattern matches only the empty
string in `Full` mode, and every string in `Search` mode. Alternation takes part
in whole-string matching even when its first alternative is shorter: `a|ab`
fully matches `ab`.

### Handling errors

Compilation returns `Result<IRegexp, iregexp::Error>`. `Error` is
`#[non_exhaustive]`, so matches need a catch-all arm:

```rust
use iregexp::{Error, IRegexp, MatchMode};

fn check(pattern: &str) -> String {
    match IRegexp::compile(pattern, MatchMode::Full) {
        Ok(_) => "valid".to_owned(),
        Err(Error::Syntax { offset, .. }) => format!("syntax error at byte {offset}"),
        Err(Error::Semantic { offset, .. }) => format!("semantic error at byte {offset}"),
        Err(Error::ResourceLimit { resource, limit }) => {
            format!("{resource} limit of {limit} exceeded")
        }
        Err(error) => format!("backend failure: {error}"),
    }
}

fn main() {
    assert_eq!(check("[a-z]+"), "valid");
    assert_eq!(check(r"\d+"), "syntax error at byte 0"); // `\d` is not I-Regexp
    assert_eq!(check("[z-a]"), "semantic error at byte 1"); // reversed range
    assert_eq!(check("a{10001}"), "range quantifier count limit of 10000 exceeded");
}
```

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

## Errors and Limits

`Error` distinguishes `Syntax`, `Semantic`, `ResourceLimit`, and `Backend`.
`Backend` reports an unexpected parser or matcher rejection, which is not
evidence that the I-Regexp is invalid. Error messages are diagnostic text, not a
stable parsing interface. Once compilation succeeds, match results are boolean.

Compilation has fixed conservative limits:

| Resource | Limit |
| --- | ---: |
| UTF-8 pattern bytes | 65,536 |
| Opening parenthesis bytes in the input | 128 |
| Numeric repetition bound | 10,000 |
| Backend syntax nesting | 256 |
| Compiled matcher size budget | 1 MiB |

The parenthesis guard counts raw `(` bytes, including escaped and class literals.
This deliberately conservative check runs before recursive parsing; it is not a
second syntax checker. A limit can therefore be reported before an invalidity
diagnostic. No process-global Pest configuration is changed. Leading zeroes
do not increase a repetition's numeric value.

These limits bound accepted compilation work; they are not an allocation quota
or a wall-clock deadline. Matching uses the `regex` crate's finite-automata engine,
whose worst-case search time scales with compiled-pattern size and text length.
The library puts no cap on input text length. Callers are responsible for
request-level input limits and concurrency budgets.

## How It Works

```text
build time   grammar/rfc9485.abnf ──abnf_to_pest──▶ Pest grammar ──pest_derive──▶ private parser
                (RFC 9485 §3)                        (Cargo OUT_DIR)

compile      pattern ─▶ size / parenthesis limits ─▶ generated parser ─▶ semantic checks
             and translation ─▶ regex::Regex (nesting and size limits) ─▶ IRegexp

match        IRegexp::is_match(text) ─▶ bool
```

`build.rs` parses the attributed [RFC grammar](grammar/rfc9485.abnf) with
`abnf_to_pest` and adds one wrapper rule, `whole = { SOI ~ i_regexp ~ EOI }`, so
parsing must consume the entire pattern. Pest then generates the private Rust
parser. The generated grammar and parser declaration are written only to Cargo's
`OUT_DIR` and carry the RFC's license notice. No generated code is checked in.

A private translation step walks the parse tree. It rejects reversed ranges and
bounds, escapes literals for `regex` syntax, maps `.` to exclude CR and LF, and
keeps Unicode categories. `Full` mode wraps the translation in absolute
`\A(?:…)\z` anchors. It does not check the span of a search result, because
alternation can pick a shorter first match.

## Testing and Provenance

The test suite asserts behavior; it does not just print observations:

- **Fixtures**: 192 main rows, of which 188 are string cases asserted in both
  modes. The other 4 specify JSONPath argument typing, and each is explicitly
  accounted for. 63 independently developed challenge rows cover matching
  outcomes. All 36 Unicode categories are checked in six positive, negated and
  class forms, in both modes.
- **Conformance**: every permitted escape inside and outside classes, scalar
  boundaries and significant whitespace, class set-operator collisions,
  multi-digit and open repetitions, reversed bounds, empty patterns, complete
  input consumption and invalid extensions.
- **Generation**: every one of the grammar's 25 ABNF rules appears in the
  generated grammar, and so does the RFC license notice.
- **Resources**: byte limits with ASCII and multibyte input, nesting depth in an
  isolated subprocess, repetition and matcher-size limits, external Pest-limit
  interference, and concurrent reuse.

This finite suite is evidence of behavior, not a proof of conformance.
[`PROVENANCE.json`](PROVENANCE.json) records the origin and SHA-256 checksum of
every imported file: RFC texts, grammar, fixtures and experiments.
`scripts/check-provenance.py` fails if any imported byte changes.

## Repository Layout

| Path | Contents |
|---|---|
| `src/lib.rs` | Public API: `IRegexp`, `MatchMode`, `Error` |
| `src/syntax.rs` | Private parsing, limits, semantic checks, translation and backend adaptation |
| `build.rs` | ABNF → Pest generation into `OUT_DIR` |
| `grammar/` | RFC 9485 ABNF and its IETF code-component license |
| `spec/` | Unmodified RFC 9485 and RFC 9535 texts |
| `tests/` | Conformance, fixture, generation and resource tests |
| `experiments/` | Historical code-generation proof and qualification adapter that predate the crate. Kept byte-for-byte for provenance; not built or maintained |
| `scripts/` | `verify.sh`, the provenance check, and CI helpers |

## Building from Source

**Prerequisites:** Rust 1.85+ and Python 3 (for the provenance check).

```bash
git clone https://github.com/strefethen/iregexp-rs.git
cd iregexp-rs
cargo test --locked
```

The full local gate needs the 1.85.0 toolchain
(`rustup toolchain install 1.85.0`):

```bash
./scripts/verify.sh
```

It runs the provenance check, `cargo fmt --check`, `cargo check`, strict Clippy
for all targets, debug and release tests, rustdoc with warnings denied, tests on
Rust 1.85, and a local packaged build.

CI runs on every push to `main` and every pull request targeting `main`:

| Job | Runners | Gates |
|---|---|---|
| Build and test | Linux, macOS, Windows | Strict Clippy, debug tests (formatting on Linux only) |
| Verify | Linux | Provenance checksums, `publish = false`, CHANGELOG version, release-mode tests, rustdoc, packaged build |
| MSRV | Linux | Debug tests on Rust 1.85 |
| Security audit | Linux | `cargo audit` against the RustSec advisory database |

Stable-toolchain jobs use a pinned Rust release, so a new Clippy lint never
turns CI red on its own. A release tag runs this whole workflow again before
anything is published.

## Project Status

iregexp-rs is pre-1.0. The public API is intentionally small, but it may change
between minor releases. Every change is recorded in [CHANGELOG.md](CHANGELOG.md).
Releases are published as [GitHub releases](https://github.com/strefethen/iregexp-rs/releases);
the crate is not published to crates.io.

To report a vulnerability, see [SECURITY.md](SECURITY.md).

## Contributing

Issues, bug reports, and feature requests are welcome.

PRs are accepted to demonstrate a fix or approach, though the maintainer may
implement changes independently after review. See [CONTRIBUTING.md](CONTRIBUTING.md)
for details.

## Acknowledgments

- [RFC 9485](https://www.rfc-editor.org/rfc/rfc9485) by Carsten Bormann and Tim
  Bray: the grammar this parser is generated from
- [pest](https://crates.io/crates/pest) / [pest_derive](https://crates.io/crates/pest_derive): parser generator
- [abnf_to_pest](https://crates.io/crates/abnf_to_pest): ABNF to Pest grammar conversion
- [regex](https://crates.io/crates/regex) / [regex-syntax](https://crates.io/crates/regex-syntax): matching engine and Unicode tables
- [serde_json](https://crates.io/crates/serde_json): fixture loading in tests

## License

The original project source is licensed under the [MIT License](LICENSE).

The RFC 9485 grammar is an IETF Code Component under the Revised BSD License
(BSD-3-Clause). Its required notice is in
[grammar/LICENSE-RFC9485.txt](grammar/LICENSE-RFC9485.txt). The parser compiled
into this library is generated from that grammar, so the crate's SPDX expression
is `MIT AND BSD-3-Clause`. Anyone redistributing a binary that includes it must
reproduce both notices. See [THIRD-PARTY.md](THIRD-PARTY.md) for full attribution.
