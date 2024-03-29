use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::{fmt::Display, fs};
use strum::IntoEnumIterator;

use super::build_element::BuildElement;
use super::Backend;
use crate::build::{Build, Target, TargetType};
use crate::compiler::Compiler;
use crate::environment::{self, Environment};
use crate::interpreter::file::File;
use crate::utils::{get_compiler_for, MachineChoice, PerMachine};

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
pub struct NinjaRule {
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
    BuildElement(BuildElement),
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
                    for l in extra.split('\n') {
                        write!(f, " ")?;
                        writeln!(f, "{}", l)?;
                    }
                }

                writeln!(f)
            }
            NinjaObject::BuildElement(build_elem) => writeln!(f, "{}", build_elem),
        }
    }
}

#[derive(Clone, Default)]
pub struct NinjaBackend {
    name: String,
    env: Environment,

    build: Build,
    build_to_src: PathBuf,
    _src_to_build: PathBuf,

    rules: Vec<NinjaObject>,
    rule_dict: HashMap<String, NinjaObject>,

    all_outputs: Vec<bool>,

    processed_targets: Vec<String>,

    build_elements: Vec<NinjaObject>,
}

impl NinjaBackend {}

impl Backend for NinjaBackend {
    fn new(env: &Environment, build: &Build) -> Self {
        let bts = pathdiff::diff_paths(&env.source_dir, &env.build_dir).unwrap_or_else(|| {
            panic!(
                "Failed to get relative path between builddir ({:?}) and sourcedir ({:?})",
                env.build_dir, env.source_dir
            )
        });
        let stb = pathdiff::diff_paths(&env.build_dir, &env.source_dir)
            .expect("Failed to get relative path between sourcedir and builddir");

        Self {
            build_to_src: bts,
            _src_to_build: stb,
            env: env.to_owned(),
            build: build.to_owned(),
            ..Default::default()
        }
    }

    fn generate(&mut self) {
        let _ninja = environment::Environment::get_ninja_command_and_version(None, None);
        let mut outfilename = self.env.build_dir.clone();
        outfilename.push("build.ninja");
        let mut tmpfilename = self.env.build_dir.clone();
        tmpfilename.push("build.ninja~");
        let mut file = fs::File::create(&tmpfilename)
            .unwrap_or_else(|_| panic!("Failed to create file {:?}", tmpfilename));

        writeln!(
            &mut file,
            "# This is the build file for project \"{}\"",
            &self.build.project_name
        )
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

        // self.build_elements = Vec::new();
        self.generate_phony();

        self.add_build_comment(NinjaObject::Comment("Build rules for targets".to_string()));
        for t in &self.build.targets.clone() {
            self.generate_target(t.1);
        }

        self.write_rules(&mut file);
        self.write_builds(&mut file);

        //
        fs::copy(&tmpfilename, outfilename).expect("Failed to copy temp nija.build to target");
        fs::remove_file(tmpfilename).expect("Failed to remove temp file");
    }

    fn get_name(&self) -> &String {
        &self.name
    }

    fn get_build_to_src(&self) -> &PathBuf {
        &self.build_to_src
    }
}

impl NinjaBackend {
    fn write_rules(&self, outfile: &mut fs::File) {
        for r in &self.rules {
            write!(outfile, "{}", r).expect("Failed to write to file");
        }
    }

    fn write_builds(&self, outfile: &mut fs::File) {
        for b in &self.build_elements {
            write!(outfile, "{}", b).expect("Failed to write build_element to output");
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

        // self.add_rule(&NinjaObject::Rule(NinjaRule::new(
        //     "CUSTOM_COMMAND_DEP",
        //     &[Command::String("$COMMAND".to_string())],
        //     &Vec::new(),
        //     "$DESC",
        //     Some("gcc"),
        //     Some("$DEPFILE"),
        //     Some("restat = 1"),
        // )));

        // self.add_rule(&NinjaObject::Rule(NinjaRule::new(
        //     "REGENERATE_BUILD",
        //     &[Command::String("".to_string())],
        //     &Vec::new(),
        //     "Regenerate build files.",
        //     None,
        //     None,
        //     None,
        // )));
    }

    fn add_build_comment(&mut self, comment: NinjaObject) {
        self.build_elements.push(comment);
    }

    fn add_rule_comment(&mut self, comment: NinjaObject) {
        self.rules.push(comment);
    }

    fn add_build(&mut self, build_elem: &mut NinjaObject) {
        if let NinjaObject::BuildElement(build) = build_elem {
            if build.rulename != "phony" {
                if self.rule_dict.contains_key(&build.rulename) {
                    if let NinjaObject::Rule(rule) = &self.rule_dict[&build.rulename] {
                        build.rule = rule.clone();
                    }
                } else {
                    warn!(
                        "build statement for {:?} references non-existent rule {}",
                        build.outfilenames, build.rulename
                    );
                }
            }
        }
        self.build_elements.push(build_elem.clone());
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
            _ => unreachable!(),
        }
    }

    fn generate_target(&mut self, target: &Target) {
        if let TargetType::BuildTarget(_) = target.target_type {
            let res = fs::create_dir_all(self.get_target_private_dir_abs(target));
            if res.is_err() {
                panic!("Failed to create target directories");
            }
        }

        // let compiled_sources = Vec::new();
        let name = target.get_id();
        if self.processed_targets.contains(&name) {
            return;
        }
        self.processed_targets.push(name);

        self.process_target_dependencies(target);

        // self.generate_shlib_aliases(&target.1, self.get_target_dir(&target.1));

        let target_sources = self.get_target_sources(target);
        // let generated_sources = self.get_target_generated_sources(target);
        // let transpiled_sources = ;

        let outname = self.get_target_filename(target);

        let mut obj_list = Vec::new();

        for src in target_sources.values() {
            let (o, _) = self.generate_single_compile(target, src);
            obj_list.push(o);
        }

        let final_obj_list = obj_list.as_slice();

        let mut elem = self.generate_link(
            target,
            &outname,
            final_obj_list,
            // linker,
            // pch_objects,
            // stdlib_args,
        );

        self.add_build(&mut elem);
    }

    fn generate_phony(&mut self) {
        self.add_build_comment(NinjaObject::Comment(String::from(
            "Phony build target, always out of date",
        )));

        let elem = BuildElement::new(
            &self.all_outputs,
            &[PathBuf::from("PHONY")],
            "phony",
            &[PathBuf::from("")],
        );

        self.add_build(&mut NinjaObject::BuildElement(elem));
    }

    // fn generate_scanner_rules(&mut self) {
    //     let rulename = "depscan";
    //     if self.rule_dict.contains_key(rulename) {
    //         return;
    //     }

    //     let mut command: Vec<Command> = Environment::get_build_command()
    //         .iter()
    //         .map(|c| Command::String(c.to_owned()))
    //         .collect();

    //     command.push(Command::String("--internal".to_owned()));
    //     command.push(Command::String("depscan".to_owned()));

    //     let args = vec![
    //         Command::String("$pickfile".to_owned()),
    //         Command::String("$out".to_owned()),
    //         Command::String("$in".to_owned()),
    //     ];
    //     let description = String::from("Module Scanner");
    //     self.add_rule(&NinjaObject::Rule(NinjaRule::new(
    //         rulename,
    //         &command,
    //         &args,
    //         &description,
    //         None,
    //         None,
    //         None,
    //     )));
    // }

    fn generate_compiler_rules(&mut self) {
        let clist = self.env.coredata.compilers.to_owned();
        for (lang, compiler) in clist {
            self.generate_compile_rules_for(&lang, &compiler);
            // self.generate_pch_rule_for(lang, compiler);
        }
    }

    fn generate_single_compile(&mut self, target: &Target, src: &File) -> (PathBuf, PathBuf) {
        let obj_basename = self.get_object_filename_from_source(target, src);
        let rel_obj = self.get_target_private_dir(target).join(obj_basename);

        let rel_src = src.rel_to_builddir(&self.build_to_src);

        let compilers: Vec<Compiler> = self.env.coredata.compilers.values().cloned().collect();
        let compiler = get_compiler_for(&compilers, src);

        let dep_file = compiler.depfile_for_object(&rel_obj);

        let mut commands = self.generate_single_compile_base_args(target, &compiler);

        commands.append(&mut self.generate_single_compile_target_args(target, &compiler));

        let mut elem = BuildElement::new(
            &self.all_outputs,
            &[rel_obj.to_owned()],
            &self.compiler_to_rulename(&compiler),
            &[rel_src.to_owned()],
        );

        elem.add_item("DEPFILE", &[dep_file.clone()]);
        elem.add_item("DEPFILE_UNQUOTED", &[dep_file]);
        elem.add_item("ARGS", &commands);

        self.add_build(&mut NinjaObject::BuildElement(elem));

        (rel_obj, rel_src)
    }

    fn generate_link(
        &self,
        target: &Target,
        outname: &PathBuf,
        obj_list: &[PathBuf], // linker: String,
                              // extra_args: Option<String>,
                              // stdlib_args: Option<String>,
    ) -> NinjaObject {
        let linker_rule = format!("cpp_LINKER{}", self.get_rule_suffix(MachineChoice::Host));
        let commands = vec![
            // "-Wl,--as-needed".to_string(),
            // "-Wl".to_string(),
            // "--no-undefined".to_string(),
        ];

        let mut elem = BuildElement::new(
            &self.all_outputs,
            vec![outname.to_owned()].as_slice(),
            &linker_rule,
            obj_list,
        );

        elem.add_item("LINK_ARGS", &commands);
        NinjaObject::BuildElement(elem)
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
        let compilers = self.env.coredata.compilers.to_owned();
        for (langname, compiler) in compilers {
            let rule = format!(
                "{}_LINKER{}",
                langname,
                self.get_rule_suffix(MachineChoice::Host)
            );

            let command: Vec<Command> = compiler
                .get_exelist()
                .iter()
                .map(|c| Command::String(c.to_owned()))
                .collect();

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
                &command,
                &args,
                &description,
                None,
                None,
                None,
            );

            self.add_rule(&NinjaObject::Rule(rule));
        }
    }

    fn generate_compile_rules_for(&mut self, lang: &str, compiler: &Compiler) {
        let rule = self.get_compiler_rule_name(lang, MachineChoice::Host);
        let command: Vec<Command> = compiler
            .get_exelist()
            .iter()
            .map(|c| Command::String(c.to_owned()))
            .collect();

        let binding = vec![
            Command::String("$ARGS".to_string()),
            Command::String("-MD -MQ $out -MF $DEPFILE -o $out -c $in".to_string()),
        ];
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

    fn generate_single_compile_base_args(
        &self,
        _target: &Target,
        compiler: &Compiler,
    ) -> Vec<String> {
        // Other things here

        Compiler::get_base_args(compiler)
    }

    fn generate_single_compile_target_args(
        &self,
        _target: &Target,
        _compiler: &Compiler,
    ) -> Vec<String> {
        Vec::new()
    }

    fn compiler_to_rulename(&self, compiler: &Compiler) -> String {
        compiler.get_compiler_rule_name(&compiler.get_language(), None)
    }

    fn process_target_dependencies(&mut self, target: &Target) {
        for t in target.get_dependencies(None) {
            if !self.processed_targets.contains(&t.get_id()) {
                self.generate_target(&t);
            }
        }
    }

    fn get_object_filename_from_source(&self, _target: &Target, source: &File) -> PathBuf {
        let mut p = String::from(&source.filename);
        p.push('.');
        p.push('o');
        PathBuf::from(p)
    }

    fn get_target_sources(&self, target: &Target) -> HashMap<PathBuf, File> {
        let mut srcs: HashMap<PathBuf, File> = HashMap::new();

        for s in &target.sources {
            let f = s.rel_to_builddir(&self.build_to_src);
            srcs.insert(f, s.to_owned());
        }

        srcs
    }

    fn get_compiler_rule_name(&self, lang: &str, machine: MachineChoice) -> String {
        format!("{}_COMPILER{}", lang, self.get_rule_suffix(machine))
    }

    fn get_rule_suffix(&self, machine: MachineChoice) -> &str {
        PerMachine::<&str>::new("_FOR_BUILD", "").getitem(machine)
    }

    fn generate_pch_rule_for(&mut self, lang: &String, compiler: &Compiler) {}

    fn get_target_private_dir_abs(&self, target: &Target) -> PathBuf {
        let mut path = self.env.build_dir.clone();
        path.push(self.get_target_private_dir(target));

        path
    }

    fn get_target_private_dir(&self, target: &Target) -> PathBuf {
        let mut path = self.get_target_filename(target);
        path.set_extension("p");
        path
    }

    fn get_target_filename(&self, target: &Target) -> PathBuf {
        let filename = match &target.target_type {
            TargetType::BuildTarget(build) => &build.filename,
            TargetType::CustomTarget => todo!(),
            TargetType::SharedLibrary => todo!(),
            TargetType::StaticLibrary => todo!(),
        };

        self.get_target_dir(target).join(filename)
    }

    fn get_target_dir(&self, target: &Target) -> PathBuf {
        PathBuf::from(&target.subdir)
    }
}

#[cfg(test)]
mod tests {

    use std::path::PathBuf;

    use crate::{backend::Backend, build::Build, environment::Environment};

    use super::NinjaBackend;

    #[test]
    fn test() {
        let env = Environment::new(&PathBuf::from("src"), &PathBuf::from("build"))
            .expect("Failed ot get environment");
        let b = Build::new(env.clone());
        let n = NinjaBackend::new(&env, &b);
        println!("{:?}", &n.build_to_src);
    }
}
