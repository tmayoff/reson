use crate::backend::ninja::NinjaBackend;
use crate::build::{BuildTarget, Target, TargetType};
use crate::compiler::Compiler;
use crate::parser::parser::Parser;
use crate::utils::MachineChoice;

use core::panic;
use std::collections::{BTreeSet, HashSet};
use std::path::PathBuf;
use std::{collections::HashMap, fs};

use super::file::File;
use super::BuiltinTypes;

use crate::{
    build::Build, environment::Environment, parser::node::Node, parser::node::NodeType,
    BUILD_FILE_NAME,
};

use super::objects::{self, ElementaryTypes, HoldableTypes, ObjectTypes};
use super::InterpreterTrait;

#[derive(Default, Clone)]
pub struct Interpreter {
    pub build: Build,
    pub environment: Environment,
    pub backend: Option<NinjaBackend>,
    // summary: HashMap<String, String>,
    // options_file: PathBuf,
    // compilers: HashMap<String, Compiler>,
    builtin: HashMap<String, BuiltinTypes>,
    subdir: String,
    // active_projectname: String,
    // subproject: String,
    // subproject_dir: String,
    // subproject_directory_name: String,
    current_lineno: i32,
    argument_depth: i32,

    ast: Option<Node>,
}

impl InterpreterTrait for Interpreter {
    fn new(
        build: Build,
        _backend: Option<String>,
        subdir: Option<&str>,
        _subproject: Option<String>,
        _subproject_dir: Option<String>,
    ) -> Result<Self, std::io::Error> {
        let mut s = Self {
            environment: build.environment.clone(),
            build,
            subdir: subdir.unwrap_or_default().to_owned(),
            // subproject: subproject.unwrap_or_default(),
            // subproject_dir: subproject_dir.unwrap_or_else(|| String::from("subprojects")),
            ..Default::default()
        };

        // s.builtin.insert(
        //     String::from("meson"),
        //     InterpreterObject::meson_main(s.build.clone(), Box::new(s.clone())),
        // );

        s.load_root_meson_file()?;
        s.sanity_check_ast();

        s.parse_project();

        s.redetect_machines();

        Ok(s)
    }

    fn get_builtin(&self) -> &HashMap<String, BuiltinTypes> {
        &self.builtin
    }

    fn run(&mut self) {
        if let Some(ast) = self.ast.clone() {
            self.evaluate_codeblock(&ast, Some(1), None);
        }
    }

    fn load_root_meson_file(&mut self) -> Result<(), std::io::Error> {
        let mut mesonfile = self.build.environment.source_dir.clone();
        mesonfile.push(self.subdir.clone());
        mesonfile.push(BUILD_FILE_NAME);

        let code = fs::read_to_string(&mesonfile).expect("Failed to read meson.build");

        let filename = String::from(
            mesonfile
                .file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default(),
        );

        self.ast = Some(Parser::new(code, filename).parse());

        Ok(())
    }

    fn parse_project(&mut self) {
        if let Some(ast) = self.ast.clone().as_ref() {
            self.evaluate_codeblock(ast, None, Some(1));
        }
    }

    fn get_sourceroot(&self) -> String {
        todo!()
    }

    fn get_funcs(&self) -> Vec<String> {
        todo!()
    }
}

impl Interpreter {
    fn redetect_machines(&mut self) {
        // todo!()
    }

    fn sanity_check_ast(&mut self) {
        assert!(self.ast.is_some());
        if let Some(ast) = &self.ast {
            assert!(
                matches!(ast.node_type, NodeType::CodeBlock { .. }),
                "AST is invalid, Possibly a bug in the parser"
            );

            // TODO check project is first function call in this file or any file in the parent dirs
            // https://github.com/mesonbuild/meson/blob/7912901accaee714fc86febdc72f4347b9397759/mesonbuild/interpreterbase/interpreterbase.py#L121
        }
    }

    fn evaluate_codeblock(&mut self, node: &Node, start: Option<usize>, end: Option<usize>) {
        let lines = match &node.node_type {
            NodeType::CodeBlock { lines } => lines,
            _ => return,
        };

        let start = start.unwrap_or(0) as usize;
        let end = end.unwrap_or(lines.len());
        let statements = &lines.as_slice()[start..end];
        for curr in statements {
            self.current_lineno = curr.lineno;
            self.evaluate_statement(curr);
        }
    }

    fn evaluate_statement(&mut self, node: &Node) -> Option<ObjectTypes> {
        match &node.node_type {
            NodeType::FunctionNode { func_name, args } => self.function_call(node, func_name, args),
            NodeType::BoolNode { value } => todo!(),
            NodeType::IDNode { value } => todo!(),
            NodeType::NumberNode { value } => todo!(),
            NodeType::StringNode { value } => Some(self.holderify(HoldableTypes::Elementary(
                ElementaryTypes::Str(value.to_owned()),
            ))),
            NodeType::FStringNode { value } => todo!(),
            NodeType::MultilineFStringNode { value } => todo!(),
            NodeType::ContinueNode => todo!(),
            NodeType::BreakNode => todo!(),
            NodeType::ArgumentNode(_) => todo!(),
            NodeType::ArrayNode { args } => todo!(),
            NodeType::DictNode { args } => todo!(),
            NodeType::EmptyNode => todo!(),
            NodeType::OrNode { left, right } => todo!(),
            NodeType::AndNode { left, right } => todo!(),
            NodeType::ComparisonNode { left, right, ctype } => todo!(),
            NodeType::ArithmeticNode {
                left,
                right,
                operation,
            } => todo!(),
            NodeType::NotNode { value } => todo!(),
            NodeType::CodeBlock { lines } => todo!(),
            NodeType::IndexNode(_) => todo!(),
            NodeType::MethodNode(_) => todo!(),
            NodeType::AssignmentNode { var_name, value } => todo!(),
            NodeType::PlusAssignmentNode { var_name, value } => todo!(),
            NodeType::ForeachClauseNode {
                varname,
                items,
                block,
            } => todo!(),
            NodeType::IfNode { condition, block } => todo!(),
            NodeType::IfClauseNode { ifs, elseblock } => todo!(),
            NodeType::UMinusNode { value } => todo!(),
            NodeType::TernaryNode {
                condition,
                trueblock,
                falseblock,
            } => todo!(),
        }
    }

    fn function_call(
        &mut self,
        node: &Node,
        func_name: &String,
        args: &Node,
    ) -> Option<ObjectTypes> {
        let (h_posargs, h_kwargs) = self.reduce_arguments(args);
        let (posargs, kwargs) = self.unholder_args(h_posargs, h_kwargs);

        let res = self.process_func(func_name.clone(), args, posargs, kwargs);

        None
    }

    fn holderify(&self, value: HoldableTypes) -> ObjectTypes {
        match value {
            HoldableTypes::Returned(r) => match r {
                objects::ReturnedObjectTypes::File(_) => todo!(),
            },
            HoldableTypes::Elementary(v) => match v {
                ElementaryTypes::Void => todo!(),
                ElementaryTypes::Bool(_) => todo!(),
                ElementaryTypes::Dict => todo!(),
                ElementaryTypes::Int(_) => todo!(),
                ElementaryTypes::List => todo!(),
                ElementaryTypes::Str(s) => ObjectTypes::Elementary(ElementaryTypes::Str(s)),
            },
            // HoldableTypes::Bool(_) => todo!(),
            // HoldableTypes::Str(str) => InterpreterObject::object_holder(HoldableTypes::Str(str)),
            // HoldableTypes::Int(_) => todo!(),
            // HoldableTypes::Dict => todo!(),
            // HoldableTypes::VecBool(_) => todo!(),
            // HoldableTypes::VecInt(_) => todo!(),
            // HoldableTypes::VecStr(_) => todo!(),
            // HoldableTypes::File => todo!(),
        }
    }

    fn unholder_args(
        &self,
        args: Vec<ObjectTypes>,
        kwargs: HashMap<String, ObjectTypes>,
    ) -> (Vec<HoldableTypes>, HashMap<String, HoldableTypes>) {
        let a = args.into_iter().map(objects::unholder).collect();
        let k = kwargs
            .into_iter()
            .map(|a| (a.0, objects::unholder(a.1)))
            .collect();

        (a, k)
    }

    fn unknown_function_call(func_name: &String) {
        panic!("Unknown function '{}'", func_name);
    }

    fn reduce_arguments(
        &mut self,
        args: &Node,
    ) -> (Vec<ObjectTypes>, HashMap<String, ObjectTypes>) {
        assert!(matches!(args.node_type, NodeType::ArgumentNode(_)));
        if let NodeType::ArgumentNode(arg_node) = &args.node_type {
            if arg_node.incorrect_order() {
                panic!("All keywords must be after positional arguments");
            }

            self.argument_depth += 1;
            let mut reduced_pos = Vec::new();
            for arg in &arg_node.arguments {
                let s = self
                    .evaluate_statement(arg)
                    .expect("At leas one value in the arguments is void");
                reduced_pos.push(s);
            }

            let mut reduced_kw: HashMap<String, ObjectTypes> = HashMap::new();
            for (key, val) in &arg_node.kwargs {
                let reduced_key = Self::key_resolver(key);
                let reduced_val = self
                    .evaluate_statement(val)
                    .expect("Value of reduced key is void.");
                reduced_kw.insert(reduced_key, reduced_val);
            }
            self.argument_depth -= 1;
            let final_kw = self.expand_default_kw(reduced_kw);

            return (reduced_pos, final_kw);
        }
        unreachable!();
    }

    fn expand_default_kw(
        &self,
        kwargs: HashMap<String, ObjectTypes>,
    ) -> HashMap<String, ObjectTypes> {
        let mut newkwargs = kwargs;
        if !newkwargs.contains_key("kwargs") {
            return newkwargs;
        }

        let to_expand = objects::unholder(newkwargs.remove("kwargs").expect("kwargs expected"));
        // assert!(matches!(to_expand, HoldableTypes::Dict));
        // assert!()
        // TODO fill this out

        newkwargs
    }

    fn key_resolver(key: &Node) -> String {
        assert!(
            matches!(key.node_type, NodeType::IDNode { .. }),
            "Invalid kwargs format"
        );

        if let NodeType::IDNode { value } = &key.node_type {
            return value.clone();
        }

        String::new()
    }

    fn process_func(
        &mut self,
        func_name: String,
        node: &Node,
        posargs: Vec<HoldableTypes>,
        kwargs: HashMap<String, HoldableTypes>,
    ) -> Option<HoldableTypes> {
        match func_name.as_str() {
            "project" => {
                self.func_project(node, posargs, kwargs);
                None
            }
            "executable" => {
                self.func_executable(node, posargs, kwargs);
                None
            }
            _ => panic!("Unknown function"),
        }
    }

    fn func_project(
        &mut self,
        _node: &Node,
        args: Vec<HoldableTypes>,
        kwargs: HashMap<String, HoldableTypes>,
    ) {
        // Kwargs used in project function
        struct ProjectKwargs {
            version: Option<String>,
            meson_version: Option<String>,
            license: Vec<String>,
            subproject_dir: String,
        }
        let mut project_args = ProjectKwargs {
            version: None,
            meson_version: None,
            license: Vec::new(),
            subproject_dir: String::new(),
        };

        assert!(
            args.len() >= 2,
            "project function requires at least 'project name' and 'language'"
        );
        assert!(matches!(
            args[0],
            HoldableTypes::Elementary(ElementaryTypes::Str(_))
        ));
        assert!(
            matches!(args[1], HoldableTypes::Elementary(ElementaryTypes::Str(_))) // || matches!(args[1], HoldableTypes::VecStr(_))
        );

        let project_name =
            if let HoldableTypes::Elementary(ElementaryTypes::Str(project_name)) = &args[0] {
                project_name.clone()
            } else {
                String::new()
            };

        let project_langs = if let HoldableTypes::Elementary(ElementaryTypes::Str(lang)) = &args[1]
        {
            vec![lang.to_owned()]
        // } else if let HoldableTypes::VecStr(langs) = &args[1] {
        //     langs.clone()
        } else {
            Vec::new()
        };

        assert!(
            !project_name.contains(':'),
            "Project name can't contain ':'"
        );

        // TODO process meson_options.txt

        self.build.project_name = project_name.clone();

        if let HoldableTypes::Elementary(ElementaryTypes::Str(v)) = &kwargs["version"] {
            project_args.version = Some(v.clone());
        }

        info!("Project Name: {}", &project_name);
        info!("Project version: {:?}", project_args.version);

        self.add_languages(&project_langs, true, MachineChoice::Host);
        self.add_languages(&project_langs, false, MachineChoice::Build);
    }

    fn func_executable(
        &mut self,
        node: &Node,
        args: Vec<HoldableTypes>,
        kwargs: HashMap<String, HoldableTypes>,
    ) {
        let mut build_target = TargetType::BuildTarget(BuildTarget::new());

        self.build_target(node, args, kwargs, &mut build_target);
    }

    fn build_target(
        &mut self,
        _node: &Node,
        args: Vec<HoldableTypes>,
        _kwargs: HashMap<String, HoldableTypes>,
        targetclass: &mut TargetType,
    ) {
        if args.is_empty() {
            panic!("Target does not have a name");
        }
        let mut sources = args;

        let holdable = sources.remove(0);
        let mut name = String::new();
        if let HoldableTypes::Elementary(ElementaryTypes::Str(n)) = holdable {
            name = n;
        }

        match targetclass {
            TargetType::BuildTarget(b) => b.filename = name.to_owned(),
            TargetType::CustomTarget => todo!(),
            TargetType::SharedLibrary => todo!(),
            TargetType::StaticLibrary => todo!(),
        }

        let files = self.source_strings_to_files(&sources);

        let target = Target::new(
            name.as_str(),
            targetclass,
            &PathBuf::from(&self.subdir),
            &files,
        );

        self.add_target(&name, target);
    }

    fn add_target(&mut self, name: &String, tobj: Target) {
        if name.is_empty() {
            panic!("Target name must not be empty");
        }

        self.validate_forbidden_targets(name);

        let idname = tobj.get_id();
        if self.build.targets.contains_key(&idname) {
            panic!(
                "Tried to create target {}, but a target of that name already exists",
                &idname
            );
        }

        self.build.targets.insert(idname, tobj);
    }

    fn validate_forbidden_targets(&self, name: &str) {
        if name.starts_with("meson-internal_") {
            panic!("Target name must not contain 'meson-internal_' are reserved");
        }
        // TODO Others
    }

    fn source_strings_to_files(&self, sources: &[HoldableTypes]) -> Vec<File> {
        let mut files = Vec::new();

        for s in sources {
            if let HoldableTypes::Elementary(ElementaryTypes::Str(source)) = s {
                files.push(File::new(source.as_str()));
            }
        }

        files
    }

    fn add_languages(
        &mut self,
        args: &[String],
        required: bool,
        for_machine: MachineChoice,
    ) -> bool {
        let success = self.add_languages_for(args, required, for_machine);
        self.redetect_machines();
        success
    }

    fn add_languages_for(
        &mut self,
        args: &[String],
        _required: bool,
        for_machine: MachineChoice,
    ) -> bool {
        let args: Vec<String> = args.iter().map(|a| a.to_lowercase()).collect();
        let mut langs: BTreeSet<String> = self
            .environment
            .coredata
            .compilers
            .keys()
            .map(|k| k.to_owned())
            .collect();
        langs.extend(args);

        // let mut success = true;
        for lang in langs {
            if self.environment.coredata.compilers.contains_key(&lang) {
                continue;
            }

            let compiler;
            let compilers = &self.environment.coredata.compilers;

            if compilers.contains_key(&lang) {
                compiler = compilers[&lang].to_owned();
            } else {
                let compiler_candidate =
                    Compiler::detect_compiler_for(&mut self.environment, &lang, &for_machine);

                if let Some(c) = compiler_candidate {
                    compiler = c;
                } else {
                    panic!("Tried to use an unknown language: {}", &lang);
                }
            }

            // Add to coredata
            self.environment
                .coredata
                .compilers
                .insert(lang.to_owned(), compiler.to_owned());
        }

        false
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn simple_test() {
        let code = r"
            project('simple', 'cpp', version: '0.1')

            executable('simple', 'main.cpp')
        "
        .to_string();

        let ast = Parser::new(code, "testfile".to_string()).parse();

        let mut inter = Interpreter {
            ast: Some(ast),
            ..Default::default()
        };

        inter.run();
    }
}
