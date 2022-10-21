use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::{fmt::Display, fs};

use strum::IntoEnumIterator;

use crate::compiler::Compiler;
use crate::environment::{self, Environment};
use crate::utils::{MachineChoice, PerMachine};

use super::Backend;

const RAW_NAMES: [&str; 6] = [
    "DEPFILE_UNQUOTED",
    "DESC",
    "pool",
    "description",
    "targetdep",
    "dyndep",
];

#[derive(Clone, Copy)]
enum Quoting {
    Both,
    NotShell,
    NotNinja,
    None,
}

impl Default for Quoting {
    fn default() -> Self {
        Quoting::Both
    }
}

#[derive(Clone, Default)]
struct NinjaCommandArg {
    arg: String,
    quoting: Quoting,
}

enum Command {
    String(String),
    CommandArg(NinjaCommandArg),
}

#[derive(Clone, Default)]
struct NinjaRule {
    pub rulename: String,
    command: Vec<NinjaCommandArg>,
    args: Vec<NinjaCommandArg>,
    description: String,
    deps: Option<String>,
    depfile: Option<String>,
    extra: Option<String>,
}

impl NinjaRule {
    fn new(
        rule: &str,
        command: &[Command],
        args: &[Command],
        description: &str,
        deps: Option<&str>,
        depfile: Option<&str>,
        extra: Option<&str>,
    ) -> Self {
        let depfile = depfile.map(|s| {
            if s == "$DEPFILE" {
                format!("{}_UNQUOTED", s)
            } else {
                s.to_string()
            }
        });

        Self {
            rulename: rule.to_owned(),
            command: command.iter().map(Self::string_to_command_arg).collect(),
            args: args.iter().map(Self::string_to_command_arg).collect(),
            description: description.to_owned(),
            deps: deps.map(|s| s.to_owned()),
            depfile,
            extra: extra.map(|s| s.to_owned()),
        }
    }

    fn string_to_command_arg(c: &Command) -> NinjaCommandArg {
        match c {
            Command::String(c) => {
                if c == "&&" {
                    NinjaCommandArg {
                        arg: c.to_owned(),
                        quoting: Quoting::NotShell,
                    }
                } else if c.starts_with('$') {
                    let reg = regex::Regex::new(r"\$\{?(\w*)\}?").expect("Failed to buld regex");
                    let group = reg.captures(c);
                    match group {
                        Some(capture) => {
                            if capture.len() > 1 && RAW_NAMES.contains(&&capture[1]) {
                                NinjaCommandArg {
                                    arg: c.to_owned(),
                                    quoting: Quoting::None,
                                }
                            } else {
                                NinjaCommandArg {
                                    arg: c.to_owned(),
                                    quoting: Quoting::NotNinja,
                                }
                            }
                        }
                        None => NinjaCommandArg {
                            arg: c.to_owned(),
                            quoting: Quoting::NotNinja,
                        },
                    }
                } else {
                    NinjaCommandArg {
                        arg: c.to_owned(),
                        quoting: Quoting::Both,
                    }
                }
            }
            Command::CommandArg(c) => c.to_owned(),
        }
    }
}

#[derive(Clone)]
enum NinjaObject {
    Comment(String),
    Rule(NinjaRule),
}

impl NinjaObject {
    fn quoter(cmd_arg: &NinjaCommandArg) -> String {
        cmd_arg.arg.to_owned()
    }
}

impl Display for NinjaObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NinjaObject::Comment(comment) => {
                for l in comment.split('\n') {
                    write!(f, "# ")?;
                    writeln!(f, "{}", l)?
                }

                writeln!(f)
            }

            NinjaObject::Rule(r) => {
                writeln!(f, "rule {}", r.rulename)?;
                let mut command: Vec<String> = r.command.iter().map(Self::quoter).collect();
                command.append(&mut r.args.iter().map(Self::quoter).collect());

                writeln!(f, " command = {}", command.join(" "))?;

                if let Some(deps) = &r.deps {
                    writeln!(f, " deps = {}", deps)?;
                }

                if let Some(depfile) = &r.depfile {
                    writeln!(f, " depfile = {}", depfile)?;
                }

                writeln!(f, " description = {}", r.description)?;

                if let Some(extra) = &r.extra {
                    for l in extra.split("\n") {
                        write!(f, " ")?;
                        writeln!(f, "{}", l)?;
                    }
                }

                writeln!(f)
            }
        }
    }
}

#[derive(Clone, Default)]
pub struct NinjaBackend {
    name: String,
    env: Environment,
    rules: Vec<NinjaObject>,
    rule_dict: HashMap<String, NinjaObject>,
}

impl NinjaBackend {
    pub fn new(env: &Environment) -> Self {
        Self {
            env: env.clone(),
            ..Default::default()
        }
    }
}

impl Backend for NinjaBackend {
    fn generate(&mut self) {
        let ninja = environment::Environment::get_ninja_command_and_version(None, None);
        let mut outfilename = self.env.build_dir.clone().unwrap_or_default();
        outfilename.push("build.ninja");
        let mut tmpfilename = self.env.build_dir.clone().unwrap_or_default();
        tmpfilename.push("build.ninja~");
        let mut file = File::create(&tmpfilename)
            .unwrap_or_else(|_| panic!("Failed to create file {:?}", tmpfilename));

        writeln!(&mut file, "# This is the build file for project \"reson\"")
            .expect("Failed to write to tmp file");
        writeln!(
            &mut file,
            "# It is autogenerated by the reson build system."
        )
        .expect("Failed to write to tmp file");
        writeln!(&mut file, "# Do not edit by hand.\n").expect("Failed to write to tmp file");
        writeln!(&mut file, "ninja_required_version = 1.8.2\n")
            .expect("Failed to write to tmp file");

        // generate rules
        self.generate_rules();

        // generate phony

        self.write_rules(&mut file);

        //
        fs::copy(&tmpfilename, outfilename).expect("Failed to copy temp nija.build to target");
        fs::remove_file(tmpfilename).expect("Failed to remove temp file");
    }

    fn get_name(&self) -> &String {
        &self.name
    }
}

impl NinjaBackend {
    fn write_rules(&self, outfile: &mut File) {
        for r in &self.rules {
            write!(outfile, "{}", r).expect("Failed to write to file");
        }
    }

    fn generate_rules(&mut self) {
        self.add_rule_comment(NinjaObject::Comment(String::from(
            "Rules for module scanning.",
        )));
        // self.generate_scanner_rules();
        self.add_rule_comment(NinjaObject::Comment(String::from("Rules for compiling.")));
        self.generate_compiler_rules();
        self.add_rule_comment(NinjaObject::Comment(String::from("Rules for linking.")));
        // self.generate_static_link_rules();
        self.generate_dynamic_link_rules();
        self.add_rule_comment(NinjaObject::Comment(String::from("Other rules")));

        self.add_rule(&NinjaObject::Rule(NinjaRule::new(
            "CUSTOM_COMMAND",
            &[Command::String("$COMMAND".to_string())],
            &Vec::new(),
            "$DESC",
            None,
            None,
            Some("restat = 1"),
        )));

        self.add_rule(&NinjaObject::Rule(NinjaRule::new(
            "CUSTOM_COMMAND_DEP",
            &[Command::String("$COMMAND".to_string())],
            &Vec::new(),
            "$DESC",
            Some("gcc"),
            Some("$DEPFILE"),
            Some("restat = 1"),
        )));

        self.add_rule(&NinjaObject::Rule(NinjaRule::new(
            "REGENERATE_BUILD",
            &[Command::String("".to_string())],
            &Vec::new(),
            "Regenerate build files.",
            None,
            None,
            None,
        )));
    }

    fn add_rule_comment(&mut self, comment: NinjaObject) {
        self.rules.push(comment);
    }

    fn add_rule(&mut self, rule: &NinjaObject) {
        match rule {
            NinjaObject::Comment(_) => unreachable!(),
            NinjaObject::Rule(nrule) => {
                if self.rule_dict.contains_key(&nrule.rulename) {
                    return;
                }
                self.rules.push(rule.to_owned());
                self.rule_dict
                    .insert(nrule.rulename.to_owned(), rule.to_owned());
            }
        }
    }

    fn generate_scanner_rules(&mut self) {
        let rulename = "depscan";
        if self.rule_dict.contains_key(rulename) {
            return;
        }

        let mut command: Vec<Command> = Environment::get_build_command()
            .iter()
            .map(|c| Command::String(c.to_owned()))
            .collect();

        command.push(Command::String("--internal".to_owned()));
        command.push(Command::String("depscan".to_owned()));

        let args = vec![
            Command::String("$pickfile".to_owned()),
            Command::String("$out".to_owned()),
            Command::String("$in".to_owned()),
        ];
        let description = String::from("Module Scanner");
        self.add_rule(&NinjaObject::Rule(NinjaRule::new(
            rulename,
            &command,
            &args,
            &description,
            None,
            None,
            None,
        )));
    }

    fn generate_compiler_rules(&mut self) {
        // for machine in MachineChoice::iter() {
        // let clist = self.env.get_coredata().get_compilers().to_owned();
        // for (lang, compiler) in &clist[&machine] {
        // if compiler.get_id() == "clang" {}
        let lang = "cpp";
        let compiler = Compiler::new(vec!["/usr/bin/clang++".to_owned()], "");
        self.generate_compile_rules_for(lang, compiler);
        // self.generate_pch_rule_for(lang, compiler);
        // }
        // }
    }

    fn generate_static_link_rules(&mut self) {
        for machine in MachineChoice::iter() {
            let static_linker = "/usr/bin/ldd";
            let rule = format!("STATIC_LINKER{}", self.get_rule_suffix(machine));
            let mut cmdlist = Vec::new();
            let args = vec![Command::String("$in".to_string())];

            cmdlist.push(Command::String(static_linker.to_string()));
            cmdlist.push(Command::String("$LINK_ARGS".to_string()));
            cmdlist.push(Command::CommandArg(NinjaCommandArg {
                arg: "$out".to_owned(),
                quoting: Quoting::None,
            }));

            let description = String::from("Linking target $out");

            self.add_rule(&NinjaObject::Rule(NinjaRule::new(
                rule.as_str(),
                &cmdlist,
                &args,
                &description,
                None,
                None,
                None,
            )));
        }
    }

    fn generate_dynamic_link_rules(&mut self) {
        // for machine in MachineChoice::iter() {
        let langname = "cpp";
        let rule = format!(
            "{}_LINKER{}",
            langname,
            self.get_rule_suffix(MachineChoice::Host)
        );
        let command = Command::String("/usr/bin/clang++".to_string());

        let args = vec![
            Command::String("$ARGS".to_string()),
            Command::CommandArg(NinjaCommandArg {
                arg: "-o $out".to_string(),
                quoting: Quoting::None,
            }),
            Command::String("$in".to_string()),
            Command::String("$LINK_ARGS".to_string()),
        ];

        let description = "Linking target $out".to_string();

        let rule = NinjaRule::new(
            rule.as_str(),
            &[command],
            &args,
            &description,
            None,
            None,
            None,
        );

        self.add_rule(&NinjaObject::Rule(rule));
        // }
    }

    fn generate_compile_rules_for(&mut self, lang: &str, compiler: Compiler) {
        let rule = self.get_compiler_rule_name(lang, MachineChoice::Host);
        let command: Vec<Command> = compiler
            .get_exelist()
            .iter()
            .map(|c| Command::String(c.to_owned()))
            .collect();

        let binding = Vec::from([Command::String("$ARGS".to_string())]);
        let args = binding.as_slice();
        let description = format!("Compiling {} object $out", "C++");
        let deps = "gcc";
        let depfile = "$DEPFILE";

        self.add_rule(&NinjaObject::Rule(NinjaRule::new(
            &rule,
            &command,
            args,
            &description,
            Some(deps),
            Some(depfile),
            None,
        )));
    }

    fn get_compiler_rule_name(&self, lang: &str, machine: MachineChoice) -> String {
        format!("{}_COMPILER{}", lang, self.get_rule_suffix(machine))
    }

    fn get_rule_suffix(&self, machine: MachineChoice) -> &str {
        PerMachine::<&str>::new("_FOR_BUILD", "").getitem(machine)
    }

    fn generate_pch_rule_for(&mut self, lang: &String, compiler: &Compiler) {}
}
