# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-09-14

The first public release: RFC 9485 I-Regexp checking and matching for Rust, with
a parser generated at build time from the RFC's own ABNF grammar.

- **Generated parser**: the RFC 9485 grammar is converted with `abnf_to_pest`
  and compiled by Pest. No lexer or grammar is maintained by hand.
- **Full and Search matching**: whole-string matching for JSONPath `match()`,
  and substring matching for `search()`.
- **Structured errors**: syntax, semantic, resource-limit, and backend failures
  are distinct `Error` variants. An invalid pattern is never reported as a
  non-match.
- **Documented resource limits** on pattern size, parenthesis count, repetition
  bounds, backend nesting, and compiled matcher size.
- **Rust 1.85**: the minimum supported Rust version.

### Added

#### Library
- `IRegexp::compile(pattern, MatchMode)` checks a pattern against the RFC 9485
  grammar and semantic restrictions. It returns a reusable, `Send + Sync`
  matcher; `IRegexp::is_match(text)` tests input against it.
- `MatchMode::Full` anchors the entire translation absolutely, so alternation
  such as `a|ab` fully matches `ab`. `MatchMode::Search` matches any substring.
- `Error::{Syntax, Semantic}` carry a byte offset. `Error::ResourceLimit` names
  the resource and its bound. `Error::Backend` reports an unexpected parser or
  matcher rejection, which is not evidence of invalid I-Regexp. `Error` is
  `#[non_exhaustive]`.
- All 36 RFC general-category names and their complements, qualified with
  regex-syntax 0.8.11 (Unicode 16.0.0). `.` excludes only CR and LF. `^` and
  `$` are literals.
- Semantic rejection of reversed character-class ranges and reversed
  repetition bounds, which the ABNF alone admits.
- Compilation limits: 65,536 UTF-8 pattern bytes, 128 raw `(` bytes, repetition
  bounds up to 10,000, backend nesting of 256, and a 1 MiB compiled matcher
  budget.

#### Quality
- Asserting tests over 192 main fixture rows and 63 independent challenge rows.
  They cover all Unicode categories in six class forms, generated-grammar
  completeness, and resource boundaries, including nesting depth in an
  isolated subprocess.
- `PROVENANCE.json` and `scripts/check-provenance.py` checksum every imported
  RFC text, grammar, fixture, and experiment.
- CI runs Clippy and tests on Linux, macOS, and Windows. Formatting,
  provenance, release-mode tests, rustdoc, MSRV, packaged-build, and
  `cargo audit` gates run on Linux. The README's Rust examples run as doctests.
- A tag-driven release workflow reruns the full CI workflow on the tagged
  commit, then publishes the matching CHANGELOG section as the GitHub release
  notes.

[0.1.0]: https://github.com/strefethen/iregexp-rs/releases/tag/v0.1.0
