use std::{env, fs, path::PathBuf};
fn main() {
    println!("cargo:rerun-if-changed=grammar.abnf");
    let source = fs::read_to_string("grammar.abnf").unwrap();
    let rules = abnf_to_pest::parse_abnf(&source).unwrap();
    let generated = format!("whole = {{ SOI ~ i_regexp ~ EOI }}\n{}\n", abnf_to_pest::render_rules_to_pest(rules).pretty(100));
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::write(out.join("generated.pest"), &generated).unwrap();
    fs::write(out.join("parser.rs"), format!("#[derive(pest_derive::Parser)]\n#[grammar_inline = {:?}]\npub struct IRegexpParser;\n", generated)).unwrap();
}
