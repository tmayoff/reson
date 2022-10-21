use std::fmt::Display;

use super::ninja::NinjaRule;

fn ninja_quote(text: &str, is_build_line: bool) -> String {
    let ninja_quote_build_pattern: regex::Regex =
        regex::Regex::new(r"[$ :\n]").expect("Failed to get regex");
    let ninja_quote_var_pattern: regex::Regex =
        regex::Regex::new(r"[$ :\n]").expect("Failed to compile regex");

    let quote_re = if is_build_line {
        ninja_quote_build_pattern
    } else {
        ninja_quote_var_pattern
    };

    if !quote_re.is_match(text) {
        return text.to_string();
    }

    if text.contains('\n') {
        panic!(
            "Ninja doesn't support newlines in rules. The content was\n {}",
            text
        );
    }

    quote_re.replace(text, r"$\g<0>").to_string()
}

#[derive(Clone, Default)]
pub struct BuildElement {
    pub rulename: String,
    pub rule: NinjaRule,

    all_outputs: Vec<bool>,
    implicit_outfilenames: Vec<String>,
    pub outfilenames: Vec<String>,
    infilenames: Vec<String>,
}

impl BuildElement {
    pub fn new(
        all_outputs: &Vec<bool>,
        outfilenames: &Vec<String>,
        rulename: &str,
        infilenames: &Vec<String>,
    ) -> Self {
        Self {
            all_outputs: all_outputs.to_owned(),
            outfilenames: outfilenames.to_owned(),
            rulename: rulename.to_string(),
            infilenames: infilenames.to_owned(),
            ..Default::default()
        }
    }
}

impl Display for BuildElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // self.check_outputs();
        let ins: Vec<String> = self
            .infilenames
            .iter()
            .map(|f| ninja_quote(f, true))
            .collect();
        let ins = ins.join(" ");

        let outs: Vec<String> = self
            .outfilenames
            .iter()
            .map(|f| ninja_quote(f, true))
            .collect();
        let outs = outs.join(" ");

        let implicit_outs: Vec<String> = self
            .implicit_outfilenames
            .iter()
            .map(|f| ninja_quote(f, true))
            .collect();
        let implicit_outs = implicit_outs.join(" ");

        let rulename = &self.rulename;
        let line = format!("build {}{}: {} {}", outs, implicit_outs, rulename, ins);
        writeln!(f, "{}", line);

        writeln!(f)
    }
}
