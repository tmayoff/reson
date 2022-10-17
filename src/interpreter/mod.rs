mod objects;

use crate::backend::NinjaBackend;
use crate::compiler::Compiler;

use core::panic;
use std::path::PathBuf;
use std::{collections::HashMap, fs};

use crate::{
    build::Build,
    environment::Environment,
    parser::node::Node,
    parser::{node::NodeKind, Parser},
    BUILD_FILE_NAME,
};

use self::objects::{HoldableTypes, InterpreterObject};

pub trait InterpreterTrait {
    fn new(
        build: Build,
        backend: Option<String>,
        subdir: Option<String>,
        subproject: Option<String>,
        subproject_dir: Option<String>,
    ) -> Result<Self, std::io::Error>
    where
        Self: std::marker::Sized;

    fn get_builtin(&self) -> &HashMap<String, InterpreterObject>;
    fn get_sourceroot(&self) -> String;
    fn get_funcs(&self) -> Vec<String>;

    fn run(&mut self);

    fn load_root_meson_file(&mut self) -> Result<(), std::io::Error>;

    fn parse_project(&mut self);
}

#[derive(Default, Clone)]
pub struct Interpreter {
    build: Build,
    environment: Environment,
    pub backend: Option<NinjaBackend>,
    coredata: String,
    summary: HashMap<String, String>,
    options_file: PathBuf,

    compilers: HashMap<String, Compiler>,
    builtin: HashMap<String, InterpreterObject>,
    subdir: String,
    active_projectname: String,
    subproject: String,
    subproject_dir: String,
    subproject_directory_name: String,

    current_lineno: i32,
    argument_depth: i32,

    ast: Option<Box<Node>>,
}

impl InterpreterTrait for Interpreter {
    fn new(
        build: Build,
        _backend: Option<String>,
        subdir: Option<String>,
        subproject: Option<String>,
        subproject_dir: Option<String>,
    ) -> Result<Self, std::io::Error> {
        let mut s = Self {
            coredata: build.environment.get_coredata(),
            environment: build.environment.clone(),
            build,
            subdir: subdir.unwrap_or_default(),
            subproject: subproject.unwrap_or_default(),
            subproject_dir: subproject_dir.unwrap_or_else(|| String::from("subprojects")),
            ..Default::default()
        };

        s.builtin.insert(
            String::from("meson"),
            InterpreterObject::meson_main(s.build.clone(), Box::new(s.clone())),
        );

        s.load_root_meson_file()?;
        s.sanity_check_ast();

        s.parse_project();

        s.redetect_machines();

        Ok(s)
    }

    fn get_builtin(&self) -> &HashMap<String, InterpreterObject> {
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

        let code = fs::read_to_string(&mesonfile)?;

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
        if let Some(ast) = self.ast.clone() {
            self.evaluate_codeblock(&ast, None, Some(1));
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
        // let machines = self.build.environment.machines;
    }

    fn sanity_check_ast(&mut self) {
        assert!(self.ast.is_some());
        if let Some(ast) = &self.ast {
            assert!(
                matches!(ast.node_kind, NodeKind::CodeBlock { .. }),
                "AST is invalid, Possibly a bug in the parser"
            );

            // TODO check project is first function call in this file or any file in the parent dirs
            // https://github.com/mesonbuild/meson/blob/7912901accaee714fc86febdc72f4347b9397759/mesonbuild/interpreterbase/interpreterbase.py#L121
        }
    }

    fn evaluate_codeblock(&mut self, node: &Node, start: Option<usize>, end: Option<usize>) {
        let lines = match &node.node_kind {
            NodeKind::CodeBlock { lines } => lines,
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

    fn evaluate_statement(&mut self, node: &Node) -> Option<InterpreterObject> {
        match &node.node_kind {
            NodeKind::FunctionNode { func_name, args } => self.function_call(node, func_name, args),
            NodeKind::BoolNode { value } => todo!(),
            NodeKind::IDNode { value } => todo!(),
            NodeKind::NumberNode { value } => todo!(),
            NodeKind::StringNode { value } => {
                Some(self.holderify(HoldableTypes::Str(value.clone())))
            }
            NodeKind::FStringNode { value } => todo!(),
            NodeKind::MultilineFStringNode { value } => todo!(),
            NodeKind::ContinueNode => todo!(),
            NodeKind::BreakNode => todo!(),
            NodeKind::ArgumentNode(_) => todo!(),
            NodeKind::ArrayNode { args } => todo!(),
            NodeKind::DictNode { args } => todo!(),
            NodeKind::EmptyNode => todo!(),
            NodeKind::OrNode { left, right } => todo!(),
            NodeKind::AndNode { left, right } => todo!(),
            NodeKind::ComparisonNode { left, right, ctype } => todo!(),
            NodeKind::ArithmeticNode {
                left,
                right,
                operation,
            } => todo!(),
            NodeKind::NotNode { value } => todo!(),
            NodeKind::CodeBlock { lines } => todo!(),
            NodeKind::IndexNode(_) => todo!(),
            NodeKind::MethodNode(_) => todo!(),
            NodeKind::AssignmentNode { var_name, value } => todo!(),
            NodeKind::PlusAssignmentNode { var_name, value } => todo!(),
            NodeKind::ForeachClauseNode {
                varname,
                items,
                block,
            } => todo!(),
            NodeKind::IfNode { condition, block } => todo!(),
            NodeKind::IfClauseNode { ifs, elseblock } => todo!(),
            NodeKind::UMinusNode { value } => todo!(),
            NodeKind::TernaryNode {
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
    ) -> Option<InterpreterObject> {
        let (h_posargs, h_kwargs) = self.reduce_arguments(args);
        let (posargs, kwargs) = self.unholder_args(h_posargs, h_kwargs);

        let res = self.process_func(func_name.clone(), args, posargs, kwargs);

        None
    }

    fn holderify(&self, value: HoldableTypes) -> InterpreterObject {
        match value {
            HoldableTypes::Boolean(_) => todo!(),
            HoldableTypes::Str(str) => InterpreterObject::object_holder(HoldableTypes::Str(str)),
            HoldableTypes::Int(_) => todo!(),
            HoldableTypes::Dict => todo!(),
            HoldableTypes::VecBool(_) => todo!(),
            HoldableTypes::VecInt(_) => todo!(),
            HoldableTypes::VecStr(_) => todo!(),
        }
    }

    fn unholder_args(
        &self,
        args: Vec<InterpreterObject>,
        kwargs: HashMap<String, InterpreterObject>,
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
    ) -> (Vec<InterpreterObject>, HashMap<String, InterpreterObject>) {
        assert!(matches!(args.node_kind, NodeKind::ArgumentNode(_)));
        if let NodeKind::ArgumentNode(arg_node) = &args.node_kind {
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

            let mut reduced_kw: HashMap<String, InterpreterObject> = HashMap::new();
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
        kwargs: HashMap<String, InterpreterObject>,
    ) -> HashMap<String, InterpreterObject> {
        let mut newkwargs = kwargs;
        if !newkwargs.contains_key("kwargs") {
            return newkwargs;
        }

        let to_expand = objects::unholder(newkwargs.remove("kwargs").expect("kwargs expected"));
        assert!(matches!(to_expand, HoldableTypes::Dict));
        // assert!()
        // TODO fill this out

        newkwargs
    }

    fn key_resolver(key: &Node) -> String {
        assert!(
            matches!(key.node_kind, NodeKind::IDNode { .. }),
            "Invalid kwargs format"
        );

        if let NodeKind::IDNode { value } = &key.node_kind {
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
                return None;
            }
            _ => panic!("Unknown function"),
        }
    }

    fn func_project(
        &mut self,
        node: &Node,
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
        assert!(matches!(args[0], HoldableTypes::Str(_)));
        assert!(
            matches!(args[1], HoldableTypes::Str(_)) || matches!(args[1], HoldableTypes::VecStr(_))
        );

        let project_name = if let HoldableTypes::Str(project_name) = &args[0] {
            project_name.clone()
        } else {
            String::new()
        };

        let project_langs = if let HoldableTypes::Str(lang) = &args[1] {
            vec![lang.clone()]
        } else if let HoldableTypes::VecStr(langs) = &args[1] {
            langs.clone()
        } else {
            Vec::new()
        };

        assert!(
            !project_name.contains(':'),
            "Project name can't contain ':'"
        );

        // TODO process meson_options.txt

        if let HoldableTypes::Str(v) = &kwargs["version"] {
            project_args.version = Some(v.clone());
        }

        info!("Project Name: {}", project_name);
        info!("Project version: {:?}", project_args.version);

        self.set_backend();
    }

    fn set_backend(&mut self) {
        if self.backend.is_some() {
            return;
        }

        self.backend = Some(NinjaBackend::new());
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::parser::Parser;

    #[test]
    fn simple_test() {
        let code = r"
            project('test_proj', 'cpp', version: '0.1.1')
        "
        .to_string();

        let ast = Parser::new(code, "testfile".to_string()).parse();

        let mut inter = Interpreter {
            ast: Some(ast),
            ..Default::default()
        };

        inter.parse_project();
    }
}
