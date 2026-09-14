# iregexp-rs

Standalone RFC 9485 I-Regexp library. The parent global agent contract applies.
Work directly in this canonical checkout; no worktrees.

## Invariants

- `grammar/rfc9485.abnf` is the extracted normative grammar. Preserve its bytes
  and attribution. Generate the parser; do not write a lexer, regex-based grammar
  approximation, or manually maintained replacement grammar.
- Generate derived parser code in Cargo OUT_DIR. Do not commit generated output.
- `spec/rfc9485.txt` owns syntax and matching semantics; its XSD references own
  semantic constraints. RFC 9535 is included only for match/search consumer context.
- Keep public types independent of Pest, regex internals, JSONPath and Arazzo.
  Use structured Result errors; invalid patterns and resource/backend failures
  must remain distinguishable. No silent fallback or degraded mode.
- Forbid unsafe code. Keep the public surface narrow and the matcher reusable.
- Tests must reject invalid syntax and semantics, preserve Unicode and both
  matching modes, and exercise documented limits. Do not change fixtures to hide
  failures; explain a fixture/spec disagreement with normative evidence.
- Retain RFC code-component license notices with the grammar and derivatives.
- `PROVENANCE.json` checksums the imported RFC texts, grammar, fixtures and
  experiments. Imported bytes stay unchanged; `.gitattributes` keeps them LF.
- Dependency requirements are caret ranges whose floors are the qualified
  versions in `Cargo.lock`. Raising a floor requires the full verification below.

## Verification

`./scripts/verify.sh` runs the provenance check, fmt, check, strict Clippy for
all targets/features, debug and release tests, rustdoc, the declared MSRV and a
local packaged build. CI (`.github/workflows/ci.yml`) runs the same gates on
Linux, macOS and Windows.

## Release

The crate is distributed from GitHub only; `publish = false` is permanent and
CI enforces it. A release bumps `version` in `Cargo.toml`, adds the matching
`## [x.y.z] - YYYY-MM-DD` section to `CHANGELOG.md`, and pushes a `vX.Y.Z` tag.
The release workflow refuses a tag without matching version and notes. Pushes,
tags and GitHub settings changes require Steve's explicit approval.
