use std::{env, error::Error, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    const GRAMMAR: &str = "grammar/rfc9485.abnf";
    const GRAMMAR_LICENSE: &str = "grammar/LICENSE-RFC9485.txt";

    println!("cargo:rerun-if-changed={GRAMMAR}");
    println!("cargo:rerun-if-changed={GRAMMAR_LICENSE}");
    let source = fs::read_to_string(GRAMMAR)?;
    let license = fs::read_to_string(GRAMMAR_LICENSE)?;
    let license_comments: String = license.lines().map(|line| format!("// {line}\n")).collect();
    let rules = abnf_to_pest::parse_abnf(&source)
        .map_err(|error| format!("could not parse {GRAMMAR}: {error:?}"))?;
    let generated = format!(
        "{license_comments}\nwhole = {{ SOI ~ i_regexp ~ EOI }}\n{}\n",
        abnf_to_pest::render_rules_to_pest(rules).pretty(100)
    );
    let output = PathBuf::from(env::var_os("OUT_DIR").ok_or("OUT_DIR is not set")?);
    fs::write(output.join("generated.pest"), &generated)?;
    fs::write(
        output.join("parser.rs"),
        format!(
            "{license_comments}\n\
             // Generated from grammar/rfc9485.abnf.\n\
             #[derive(pest_derive::Parser)]\n\
             #[grammar_inline = {generated:?}]\n\
             pub(crate) struct IRegexpParser;\n"
        ),
    )?;
    Ok(())
}
