# RFC 9485 grammar code-generation proof

This isolated experiment converted RFC 9485 Section 3 Figure 1 directly from ABNF to Pest using abnf_to_pest 0.6.0, then compiled a Rust parser with pest_derive 2.9.1. No handwritten lexer or parser was added.

The RFC grammar is 37 lines / 25 rules. The generator performs ABNF conversion and identifier escaping. The only manually added grammar rule requires the entire input:

```pest
whole = { SOI ~ i_regexp ~ EOI }
```

The generated 26-rule Pest grammar is saved in `generated.pest`. `build.rs` regenerates it and the Rust derive declaration under Cargo's OUT_DIR. `grammar.abnf` is the extracted specification; only common publication indentation was removed. No grammar productions were rewritten.

## Result

**21 focused syntax cases matched expectations. Two additional cases deliberately demonstrate required semantic checks.** This is not 23 full-conformance passes.

- RFC 7405 `%s` case-sensitive strings work: `\p{Lu}` and `\P{Lu}` parse, `\p{lu}` rejects.
- `%xE000-10FFFF` works, including astral Unicode and U+10FFFF.
- `1*%x30-39` generates one-or-more digits; `a{12}` and `a{12,34}` parse.
- Whole-input validation rejects trailing junk. Classes, trailing/leading dashes, complements, and grouped alternatives passed the focused cases.
- `[z-a]` and `a{3,2}` parse because their ordering constraints are not expressed in the ABNF. Semantic validation must reject them separately.

`proof.json` contains every observation, and `provenance.json` records exact versions, source hashes, toolchain, and limits. The generated parser recognizes pattern syntax; it does not generate regex matching/translation semantics. General ABNF/PEG language equivalence and production resource limits were not qualified here.

## Reproduce

From this directory, with Rust installed:

```sh
cargo run --locked --quiet > rerun.json
```

Cargo downloads the pinned dependencies on a first run. With the qualification session's populated disposable cache, the exact offline command is:

```sh
CARGO_HOME=/private/tmp/arazzo-rust-jsonpath-review/cargo-home \
cargo run --offline --locked --quiet > rerun.json
```

The original observed command was `cargo run --quiet`, with that disposable cache; `Cargo.lock` is the resulting captured lockfile. No arazzo-cli code or the prior qualification bundle was changed.

## Attribution

`grammar.abnf` and its generated derivative come from IETF RFC 9485 by Carsten Bormann and Tim Bray. The complete required code-component notice is in `LICENSE-RFC9485.txt`. abnf_to_pest is MIT OR Apache-2.0 licensed; dependencies are resolved from the pinned Cargo manifest/lockfile, not vendored in this proof.
