mod objects;

use core::panic;
use std::{collections::HashMap, fs};

use crate::{
    build::Build,
    parser::node::Node,
    parser::{node::NodeKind, Parser},
    BUILD_FILE_NAME,
};

use self::objects::{HoldableTypes, InterpreterObject};

type FuncType =
    Box<dyn Fn(&Node, Vec<HoldableTypes>, HashMap<String, HoldableTypes>) -> Option<HoldableTypes>>;

#[derive(Default)]
pub struct Interpreter {
    build: Build,

    subdir: String,
    subproject: String,
    subproject_dir: String,

    current_lineno: i32,
    argument_depth: i32,

    funcs: HashMap<String, FuncType>,

    ast: Option<Box<Node>>,
}

impl Interpreter {
    pub fn new(
        build: Build,
        _backend: Option<String>,
        subdir: Option<String>,
        subproject: Option<String>,
        subproject_dir: Option<String>,
    ) -> Result<Self, std::io::Error> {
        let mut s = Self {
            build,
            subdir: subdir.unwrap_or_default(),
            subproject: subproject.unwrap_or_default(),
            subproject_dir: subproject_dir.unwrap_or_else(|| String::from("subprojects")),
            current_lineno: 0,
            ast: None,
            ..Default::default()
        };

        s.load_root_meson_file()?;
        s.sanity_check_ast();

        s.parse_project();

        Ok(s)
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

    fn parse_project(&mut self) {
        if let Some(ast) = self.ast.clone() {
            self.evaluate_codeblock(ast, None, Some(1));
        }
    }

    pub fn run(&mut self) {
        //
    }

    fn evaluate_codeblock(&mut self, node: Box<Node>, start: Option<usize>, end: Option<usize>) {
        let lines = if let NodeKind::CodeBlock { lines } = node.node_kind {
            lines
        } else {
            Vec::new()
        };

        let start = start.unwrap_or(0) as usize;
        let end = end.unwrap_or(lines.len() - 1);
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

        if self.funcs.contains_key(func_name) {
            let func = self.funcs.get(func_name).expect("Expected function");
            let func_args = posargs;
            let res = (func)(node, func_args, kwargs);
            res.map(|r| self.holderify(r))
        } else {
            Self::unknown_function_call(func_name);
            None
        }
    }

    fn holderify(&self, value: HoldableTypes) -> InterpreterObject {
        match value {
            HoldableTypes::Boolean(_) => todo!(),
            HoldableTypes::Str(str) => InterpreterObject::object_holder(HoldableTypes::Str(str)),
            HoldableTypes::Int(_) => todo!(),
            HoldableTypes::Dict => todo!(),
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
