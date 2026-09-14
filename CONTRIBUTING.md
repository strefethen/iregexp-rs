# Contributing to iregexp-rs

## Bug Reports

Bug reports are welcome. Please include:

- The pattern, the `MatchMode`, and the input text
- Expected vs. actual behavior. For a matching or validity disagreement, cite
  the RFC 9485 section (or the XSD rule it adopts) that supports the expected
  result.
- The full `Error` value (`{:?}`) when compilation fails unexpectedly
- Environment details: OS, Rust version, and the iregexp-rs tag or commit

A minimal Rust snippet that reproduces the issue is ideal:

```rust
use iregexp::{IRegexp, MatchMode};

fn main() {
    let result = IRegexp::compile("your-pattern", MatchMode::Full);
    println!("{result:?}");
}
```

Normative behavior comes from [`spec/rfc9485.txt`](spec/rfc9485.txt), not from
other regex engines. A difference from PCRE, ECMAScript, or the Rust `regex`
crate is expected whenever I-Regexp defines the construct differently.

## Security Issues

Please don't report vulnerabilities in public issues. See [SECURITY.md](SECURITY.md).

## Pull Requests

Feel free to open issues, and PRs too if you want to show a proposed fix in
action. Just know that I won't merge them directly. I'll review what's there
and decide independently whether and how to address it. PRs may be closed
without detailed feedback.

This is a single-maintainer project. I don't have the bandwidth to review
external code, and I'm accountable for everything it ships. I hope that's
understandable.

If you do open a PR, run the same gates as CI first:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all-features --locked
```

`./scripts/verify.sh` runs the complete local suite, including the provenance
check and the Rust 1.85 MSRV build. Please don't modify `grammar/`, `spec/`, or
`tests/fixtures/`. Their bytes are checksummed in `PROVENANCE.json`, and a
fixture/spec disagreement should be explained with normative evidence rather
than an edited fixture.
