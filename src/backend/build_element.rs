use std::{fmt::Display, path::PathBuf};

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
    pub outfilenames: Vec<PathBuf>,

    // all_outputs: Vec<bool>,
    implicit_outfilenames: Vec<PathBuf>,
    infilenames: Vec<PathBuf>,
    elems: Vec<(String, Vec<String>)>,
}

impl BuildElement {
    pub fn new(
        _all_outputs: &[bool],
        outfilenames: &[PathBuf],
        rulename: &str,
        infilenames: &[PathBuf],
    ) -> Self {
        Self {
            // all_outputs: all_outputs.to_owned(),
            outfilenames: outfilenames.to_owned(),
            rulename: rulename.to_string(),
            infilenames: infilenames.to_owned(),
            ..Default::default()
        }
    }

    pub fn add_item(&mut self, name: &str, elems: &[String]) {
        self.elems.push((name.to_owned(), elems.to_owned()));
    }
}

impl Display for BuildElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ins: Vec<String> = self
            .infilenames
            .iter()
            .map(|f| ninja_quote(f.to_str().unwrap_or_default(), true))
            .collect();
        let ins = ins.join(" ");

        let outs: Vec<String> = self
            .outfilenames
            .iter()
            .map(|f| ninja_quote(f.to_str().unwrap_or_default(), true))
            .collect();
        let outs = outs.join(" ");

        let implicit_outs: Vec<String> = self
            .implicit_outfilenames
            .iter()
            .map(|f| ninja_quote(f.to_str().unwrap_or_default(), true))
            .collect();
        let implicit_outs = implicit_outs.join(" ");

        let rulename = &self.rulename;
        let line = format!("build {}{}: {} {}", outs, implicit_outs, rulename, ins);
        writeln!(f, "{}", line)?;

        for e in &self.elems {
            let (name, elems) = e;
            let mut line = format!(" {} = ", name);
            let mut newelems = Vec::new();
            for i in elems {
                newelems.push(ninja_quote(i, false));
            }

            line.push_str(&newelems.join(" "));
            writeln!(f, "{}", line)?;
        }

        writeln!(f)
    }
}
