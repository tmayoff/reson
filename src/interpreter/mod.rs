pub mod file;
mod functions;
mod objects;

use crate::build::Target;
use crate::compiler::Compiler;
use crate::parser::parser::Parser;
use crate::utils::MachineChoice;
use crate::{backend::ninja::NinjaBackend, build::TargetType};
use crate::{build::Build, environment::Environment, parser::node::Node, BUILD_FILE_NAME};
use file::File;
use objects::{unholder, ElementaryTypes, Object};
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::rc::Rc;
use std::{collections::HashMap, fs};

#[derive(Default, Clone)]
pub struct Interpreter {
    pub build: Build,
    pub environment: Environment,
    pub backend: Option<NinjaBackend>,
    // summary: HashMap<String, String>,
    // options_file: PathBuf,
    // compilers: HashMap<String, Compiler>,
    subdir: String,
    // active_projectname: String,
    // subproject: String,
    // subproject_dir: String,
    // subproject_directory_name: String,
    argument_depth: i32,

    ast: Option<Node>,

    variables: HashMap<String, Object>,
}

impl Interpreter {
    pub fn new(
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

    pub fn run(&mut self) {
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

        self.ast = Some(Parser::new(&code, &filename).parse());

        Ok(())
    }

    fn parse_project(&mut self) {
        if let Some(ast) = self.ast.clone().as_ref() {
            self.evaluate_codeblock(ast, None, Some(1));
        }
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
                matches!(ast, Node::CodeBlock { .. }),
                "AST is invalid, Possibly a bug in the parser"
            );

            // TODO check project is first function call in this file or any file in the parent dirs
            // https://github.com/mesonbuild/meson/blob/7912901accaee714fc86febdc72f4347b9397759/mesonbuild/interpreterbase/interpreterbase.py#L121
        }
    }

    fn evaluate_codeblock(&mut self, node: &Node, start: Option<usize>, end: Option<usize>) {
        let lines = match &node {
            Node::CodeBlock { lines } => lines,
            _ => return,
        };

        let start = start.unwrap_or(0);
        let end = end.unwrap_or(lines.len());
        let statements = &lines.as_slice()[start..end];
        for curr in statements {
            //            self.current_lineno = curr.lineno;
            self.evaluate_statement(curr);
        }
    }

    fn evaluate_statement(&mut self, node: &Node) -> Option<Object> {
        match &node {
            Node::Function { func_name, args } => self.function_call(node, func_name, args),
            Node::BoolNode { value } => {
                Some(self.holderify(Object::Elementary(ElementaryTypes::Bool(value.to_owned()))))
            }
            Node::ID { value } => todo!(),
            Node::Number { value } => {
                Some(self.holderify(Object::Elementary(ElementaryTypes::Int(value.to_owned()))))
            }
            Node::String { value } => {
                Some(self.holderify(Object::Elementary(ElementaryTypes::Str(value.to_owned()))))
            }
            Node::FStringNode { value } => todo!(),
            Node::MultilineFStringNode { value } => todo!(),
            Node::ContinueNode => todo!(),
            Node::BreakNode => todo!(),
            Node::Argument(_) => todo!(),
            Node::Array { args } => self.evaluate_arraystatement(args),
            Node::Dict { args } => self.evaluate_dictstatement(args),
            Node::Empty => todo!(),
            Node::OrNode { left, right } => todo!(),
            Node::AndNode { left, right } => todo!(),
            Node::Comparison { left, right, ctype } => todo!(),
            Node::Arithmetic {
                left,
                right,
                operation,
            } => todo!(),
            Node::NotNode { value } => todo!(),
            Node::CodeBlock { lines } => todo!(),
            Node::Index(_) => todo!(),
            Node::Method(method) => self.method_call(method),
            Node::Assignment { var_name, value } => {
                self.assignment(&var_name, value);
                None
            }
            Node::PlusAssignmentNode { var_name, value } => todo!(),
            Node::ForeachClauseNode {
                varname,
                items,
                block,
            } => todo!(),
            Node::IfNode { condition, block } => todo!(),
            Node::IfClauseNode { ifs, elseblock } => todo!(),
            Node::UMinusNode { value } => todo!(),
            Node::Ternary {
                condition,
                trueblock,
                falseblock,
            } => todo!(),
        }
    }

    fn evaluate_arraystatement(&mut self, args: &Node) -> Option<Object> {
        let (args, kwargs) = self.reduce_arguments(args);
        if !kwargs.is_empty() {
            panic!("Keywork arguments are invalid in array construction");
        }

        let args = args.iter().map(unholder).collect();

        Some(Object::Elementary(ElementaryTypes::List(args)))
    }

    fn evaluate_dictstatement(&mut self, args: &Node) -> Option<Object> {
        Some(Object::Elementary(ElementaryTypes::Dict))
    }

    fn holderify(&self, value: Object) -> Object {
        match value {
            Object::Elementary(v) => match v {
                // ElementaryTypes::Void => todo!(),
                ElementaryTypes::Bool(b) => Object::Elementary(ElementaryTypes::Bool(b)),
                ElementaryTypes::Dict => todo!(),
                ElementaryTypes::Int(i) => Object::Elementary(ElementaryTypes::Int(i)),
                ElementaryTypes::List(_) => todo!(),
                ElementaryTypes::Str(s) => Object::Elementary(ElementaryTypes::Str(s)),
            },
            Object::BuiltinTypes => todo!(),
        }
    }

    fn unholder_args(
        &self,
        args: Vec<Object>,
        kwargs: HashMap<String, Object>,
    ) -> (Vec<ElementaryTypes>, HashMap<String, ElementaryTypes>) {
        let a = args.into_iter().map(|a| objects::unholder(&a)).collect();

        let k = kwargs
            .into_iter()
            .map(|a| (a.0, objects::unholder(&a.1)))
            .collect();

        (a, k)
    }

    fn reduce_arguments(&mut self, args: &Node) -> (Vec<Object>, HashMap<String, Object>) {
        assert!(matches!(args, Node::Argument(_)));
        if let Node::Argument(arg_node) = &args {
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

            let mut reduced_kw: HashMap<String, Object> = HashMap::new();
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

    fn expand_default_kw(&self, kwargs: HashMap<String, Object>) -> HashMap<String, Object> {
        let newkwargs = kwargs;
        if !newkwargs.contains_key("kwargs") {
            return newkwargs;
        }

        // let to_expand = objects::unholder(newkwargs.remove("kwargs").expect("kwargs expected"));
        // assert!(matches!(to_expand, HoldableTypes::Dict));
        // assert!()
        // TODO fill this out

        newkwargs
    }

    fn key_resolver(key: &Node) -> String {
        assert!(matches!(key, Node::ID { .. }), "Invalid kwargs format");

        if let Node::ID { value } = key {
            return value.clone();
        }

        String::new()
    }

    fn assignment(&mut self, var_name: &str, value: &Rc<Node>) {
        let variable = self
            .evaluate_statement(value)
            .expect("Variable must be assigned to an actual value");

        self.set_variable(var_name, variable);
    }

    fn set_variable(&mut self, var_name: &str, variable: Object) {
        self.variables.insert(var_name.to_string(), variable);
    }

    fn get_variable(&self, var_name: &str) -> Option<Object> {
        self.variables.get(var_name).cloned()
    }

    fn build_target(
        &mut self,
        _node: &Node,
        args: Vec<ElementaryTypes>,
        _kwargs: HashMap<String, ElementaryTypes>,
        targetclass: &mut TargetType,
    ) {
        if args.is_empty() {
            panic!("Target does not have a name");
        }
        let mut sources = args;

        let holdable = sources.remove(0);
        let name = if let ElementaryTypes::Str(n) = holdable {
            n
        } else {
            String::new()
        };

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

        if self.build.targets.contains_key(name) {
            panic!(
                "Tried to create target {}, but a target of that name already exists",
                name
            );
        }

        self.build.targets.insert(name.to_string(), tobj);
    }

    fn validate_forbidden_targets(&self, name: &str) {
        if name.starts_with("meson-internal_") {
            panic!("Target name must not contain 'meson-internal_' are reserved");
        }
        // TODO Others
    }

    fn source_strings_to_files(&self, sources: &[ElementaryTypes]) -> Vec<File> {
        let mut files = Vec::new();

        for s in sources {
            if let ElementaryTypes::Str(source) = s {
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

    use std::path::Path;

    use super::*;

    fn run_interpreter(inter: &mut Interpreter) {
        inter.sanity_check_ast();
        inter.parse_project();
        inter.redetect_machines();
        inter.run();
    }

    #[test]
    fn simple_test() {
        let code = r"
            project('simple', 'cpp', version: '0.1')
        ";

        let ast = Parser::new(code, "testfile").parse();

        let env = Environment::new(Path::new("."), Path::new(".")).unwrap();
        let build = Build::new(env.clone());
        let mut inter = Interpreter {
            ast: Some(ast),
            environment: env,
            build,
            ..Default::default()
        };

        run_interpreter(&mut inter);

        assert_eq!(&inter.build.project_name, "simple");
        assert!(inter.environment.coredata.compilers.contains_key("cpp"));
    }

    #[test]
    fn multiple_langs_test() {
        let code = r"
            project('simple', ['cpp', 'cpp'], version: '0.1')
        ";

        let ast = Parser::new(code, "testfile").parse();

        let env = Environment::new(Path::new("."), Path::new(".")).unwrap();
        let build = Build::new(env.clone());
        let mut inter = Interpreter {
            ast: Some(ast),
            environment: env,
            build,
            ..Default::default()
        };

        run_interpreter(&mut inter);

        assert_eq!(&inter.build.project_name, "simple");
        assert!(inter.environment.coredata.compilers.contains_key("cpp"));
    }

    #[test]
    fn vars_test() {
        let code = r"
            project('simple', ['cpp'], version: '0.1')

            a = 'Hello World'
            b = true
            c = 100
            d = ['Hello World', 1]
            e = {'Hello': 'World'}
        ";

        let ast = Parser::new(code, "testfile").parse();

        let env = Environment::new(Path::new("."), Path::new(".")).unwrap();
        let build = Build::new(env.clone());
        let mut inter = Interpreter {
            ast: Some(ast),
            environment: env,
            build,
            ..Default::default()
        };

        let expected = HashMap::from([
            (
                "a",
                Object::Elementary(ElementaryTypes::Str(String::from("Hello World"))),
            ),
            ("b", Object::Elementary(ElementaryTypes::Bool(true))),
            ("c", Object::Elementary(ElementaryTypes::Int(100))),
            (
                "d",
                Object::Elementary(ElementaryTypes::List(vec![
                    ElementaryTypes::Str(String::from("Hello World")),
                    ElementaryTypes::Int(1),
                ])),
            ),
            ("e", Object::Elementary(ElementaryTypes::Dict)),
        ]);

        run_interpreter(&mut inter);

        assert_eq!(inter.variables.len(), 5);
        for t in expected {
            assert!(inter.variables.contains_key(t.0));
            assert_eq!(inter.variables.get(t.0).unwrap().to_owned(), t.1);
        }
    }

    #[test]
    fn bool_test() {
        let code = r"
        project('simple', ['cpp'], version: '0.1')

        a = true
        b = a.to_int()
        c = a.to_string()
    ";

        let ast = Parser::new(code, "testfile").parse();

        let env = Environment::new(Path::new("."), Path::new(".")).unwrap();
        let build = Build::new(env.clone());
        let mut inter = Interpreter {
            ast: Some(ast),
            environment: env,
            build,
            ..Default::default()
        };

        let expected = HashMap::from([
            ("a", Object::Elementary(ElementaryTypes::Bool(true))),
            ("b", Object::Elementary(ElementaryTypes::Int(1))),
            (
                "c",
                Object::Elementary(ElementaryTypes::Str(String::from("true"))),
            ),
        ]);

        run_interpreter(&mut inter);

        assert_eq!(inter.variables.len(), 3);
        for t in expected {
            assert!(inter.variables.contains_key(t.0));
            assert_eq!(inter.variables.get(t.0).unwrap().to_owned(), t.1);
        }
    }

    #[test]
    fn int_test() {
        let code = r"
        project('simple', ['cpp'], version: '0.1')

        a = 1
        b = a.is_even()
        c = a.is_odd()
        d = a.to_string()
    ";

        let ast = Parser::new(code, "testfile").parse();

        let env = Environment::new(Path::new("."), Path::new(".")).unwrap();
        let build = Build::new(env.clone());
        let mut inter = Interpreter {
            ast: Some(ast),
            environment: env,
            build,
            ..Default::default()
        };

        let expected = HashMap::from([
            ("a", Object::Elementary(ElementaryTypes::Int(1))),
            ("b", Object::Elementary(ElementaryTypes::Bool(false))),
            ("c", Object::Elementary(ElementaryTypes::Bool(true))),
            (
                "d",
                Object::Elementary(ElementaryTypes::Str(String::from("1"))),
            ),
        ]);

        run_interpreter(&mut inter);

        assert_eq!(inter.variables.len(), 4);
        for t in expected {
            assert!(inter.variables.contains_key(t.0));
            assert_eq!(inter.variables.get(t.0).unwrap().to_owned(), t.1);
        }
    }

    #[test]
    fn str_test() {
        let code = r"
        project('simple', ['cpp'], version: '0.1')

        a = 'Hello World'
        b = a.contains('Hello')
        c = a.endswith('Hello')
        d = ' '.join(['Hello', 'World'])
        e = a.replace('Hello', 'Bye')
        f = a.split(' ')
        g = a.startswith('Hello')
        h = a.strip()
        i = a.strip(['e', 'o'])
        j = a.substring(5)
        k = a.substring(5, 7)
        l = a.substring(1, -1)

        integer = '100'
        m = integer.to_int()

        n = a.to_lower()
        o = a.to_upper()
        p = a.underscorify()
    ";

        let ast = Parser::new(code, "testfile").parse();

        let env = Environment::new(Path::new("."), Path::new(".")).unwrap();
        let build = Build::new(env.clone());
        let mut inter = Interpreter {
            ast: Some(ast),
            environment: env,
            build,
            ..Default::default()
        };

        let expected = HashMap::from([
            (
                "a",
                Object::Elementary(ElementaryTypes::Str(String::from("Hello World"))),
            ),
            ("b", Object::Elementary(ElementaryTypes::Bool(true))),
            ("c", Object::Elementary(ElementaryTypes::Bool(false))),
            (
                "d",
                Object::Elementary(ElementaryTypes::Str(String::from("Hello World"))),
            ),
            (
                "e",
                Object::Elementary(ElementaryTypes::Str(String::from("Bye World"))),
            ),
            (
                "f",
                Object::Elementary(ElementaryTypes::List(vec![
                    ElementaryTypes::Str(String::from("Hello")),
                    ElementaryTypes::Str(String::from("World")),
                ])),
            ),
            ("g", Object::Elementary(ElementaryTypes::Bool(true))),
            (
                "h",
                Object::Elementary(ElementaryTypes::Str(String::from("HelloWorld"))),
            ),
            (
                "i",
                Object::Elementary(ElementaryTypes::Str(String::from("Hll Wrld"))),
            ),
            (
                "j",
                Object::Elementary(ElementaryTypes::Str(String::from("Hello"))),
            ),
            (
                "k",
                Object::Elementary(ElementaryTypes::Str(String::from(" W"))),
            ),
            (
                "l",
                Object::Elementary(ElementaryTypes::Str(String::from("ello Worl"))),
            ),
            (
                "integer",
                Object::Elementary(ElementaryTypes::Str(String::from("100"))),
            ),
            ("m", Object::Elementary(ElementaryTypes::Int(100))),
            (
                "n",
                Object::Elementary(ElementaryTypes::Str(String::from("hello world"))),
            ),
            (
                "o",
                Object::Elementary(ElementaryTypes::Str(String::from("HELLO WORLD"))),
            ),
            (
                "p",
                Object::Elementary(ElementaryTypes::Str(String::from("Hello_World"))),
            ),
        ]);

        run_interpreter(&mut inter);

        assert_eq!(inter.variables.len(), expected.len());
        for t in expected {
            assert!(inter.variables.contains_key(t.0));
            assert_eq!(inter.variables.get(t.0).unwrap().to_owned(), t.1);
        }
    }
}
