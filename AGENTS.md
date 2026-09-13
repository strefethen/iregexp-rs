# iregexp-rs

Standalone RFC 9485 I-Regexp library. Read HANDOFF.md before implementation.
The parent global agent contract applies. Work directly in this canonical
checkout; no worktrees. No other repository may be changed by this initiative.

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

## Verification

Use Cargo check, fmt --check, strict Clippy for all targets/features, tests,
release tests where resource behavior can differ, and rustdoc. Verify the declared
MSRV and a local packaged build. Publication, pushes and remote creation remain
outside this local implementation scope. The continuation task follows the
global candidate/review workflow in this repository. Obtain fresh independent
review of the exact candidate before claiming the library ready.
