# Source attribution

The original library and verification code are under [MIT](LICENSE-MIT).
The extracted RFC 9485 grammar is an IETF code component under the Revised
BSD License in [grammar/LICENSE-RFC9485.txt](grammar/LICENSE-RFC9485.txt).
The build retains that notice in the generated grammar and parser declaration.

`PROVENANCE.json` records the original paths and SHA-256 checksums of the
imported RFC texts, normative grammar, exploratory proof, adapter and fixtures.
Those imported bytes are retained unchanged. The experiments record prior
qualification; the root crate and asserting tests implement the reusable library.

RFC 9485 section 4 adopts XSD matching semantics. Semantic constraints are
grounded in [XSD 1.0 character ranges](https://www.w3.org/TR/2004/REC-xmlschema-2-20041028/#char-ranges)
and [XSD 1.1 regular expressions](https://www.w3.org/TR/2012/REC-xmlschema11-2-20120405/#regexs).

Cargo dependencies retain their own licenses. `cargo tree --locked` reports the
complete pinned resolution used by this checkout.
