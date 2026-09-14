# Source attribution

## Project source

The original library, build script, tests, fixtures and verification code are
under the [MIT License](LICENSE).

## RFC 9485 grammar

[`grammar/rfc9485.abnf`](grammar/rfc9485.abnf) is RFC 9485 Section 3, Figure 1,
by Carsten Bormann and Tim Bray. It is an IETF Code Component under the Revised
BSD License (BSD-3-Clause), whose full notice is in
[`grammar/LICENSE-RFC9485.txt`](grammar/LICENSE-RFC9485.txt). The build copies that
notice into the generated grammar and parser declaration in Cargo `OUT_DIR`.
Because the generated parser is compiled into the library, the crate's license
expression is `MIT AND BSD-3-Clause`. Binary redistributions must reproduce both
notices.

## RFC texts

[`spec/rfc9485.txt`](spec/rfc9485.txt) and [`spec/rfc9535.txt`](spec/rfc9535.txt)
are byte-identical to the documents published at
<https://www.rfc-editor.org/rfc/rfc9485.txt> and
<https://www.rfc-editor.org/rfc/rfc9535.txt>. They are reproduced unmodified
under BCP 78 and the IETF Trust Legal Provisions. Copyright is held by the IETF
Trust and the document authors.

RFC 9485 section 4 adopts XSD matching semantics. Semantic constraints are
grounded in [XSD 1.0 character ranges](https://www.w3.org/TR/2004/REC-xmlschema-2-20041028/#char-ranges)
and [XSD 1.1 regular expressions](https://www.w3.org/TR/2012/REC-xmlschema11-2-20120405/#regexs).

## Imported files and experiments

`PROVENANCE.json` records the origin and SHA-256 checksum of every imported file:
the RFC texts, normative grammar, the code-generation proof and qualification
adapter under `experiments/`, and the test fixtures. Those bytes are retained
unchanged, and `scripts/check-provenance.py` verifies them. The experiments
are historical records of qualification work that predates the crate. They
are not built or maintained, and they may reference crates, data or paths
that exist only in their original workspace. The root crate and its
asserting tests implement the reusable library.

## Cargo dependencies

Cargo dependencies keep their own licenses and are not vendored. They are
`pest`, `pest_derive`, `regex` and `regex-syntax`, plus `abnf_to_pest` at build
time and `serde_json` for tests. `cargo tree --locked` reports the complete
resolution this repository is tested with.
