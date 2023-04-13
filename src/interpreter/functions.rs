use super::{
    objects::{ElementaryTypes, Object},
    Interpreter,
};
use crate::{
    build::{BuildTarget, TargetType},
    parser::node::{MethodNode, Node},
    utils::MachineChoice,
};
use std::{borrow::Borrow, collections::HashMap};

impl Interpreter {
    pub fn method_call(&mut self, method: &MethodNode) -> Option<Object> {
        let invocable = &method.source_object;

        let obj = if let Node::ID { value } = invocable.borrow() {
            self.get_variable(value)
        } else {
            self.evaluate_statement(invocable)
        };

        let obj = obj.unwrap();

        let method_name = &method.name;

        let (h_posargs, h_kwargs) = self.reduce_arguments(&method.args);
        let (posargs, kwargs) = self.unholder_args(h_posargs, h_kwargs);

        Some(obj.method_call(method_name, posargs))
    }

    pub fn function_call(&mut self, _node: &Node, func_name: &str, args: &Node) -> Option<Object> {
        let (h_posargs, h_kwargs) = self.reduce_arguments(args);
        let (posargs, kwargs) = self.unholder_args(h_posargs, h_kwargs);

        self.process_func(func_name, args, posargs, kwargs)
    }

    fn process_func(
        &mut self,
        func_name: &str,
        node: &Node,
        posargs: Vec<ElementaryTypes>,
        kwargs: HashMap<String, ElementaryTypes>,
    ) -> Option<Object> {
        match func_name {
            "project" => {
                self.func_project(node, posargs, kwargs);
                Some(Object::Elementary(ElementaryTypes::Void))
            }
            "executable" => self.func_executable(node, posargs, kwargs),
            "library" => self.func_library(),
            _ => panic!("Unknown function {}", func_name),
        }
    }

    fn func_project(
        &mut self,
        _node: &Node,
        args: Vec<ElementaryTypes>,
        kwargs: HashMap<String, ElementaryTypes>,
    ) {
        // TODO Fill this out
        // Kwargs used in project function
        struct ProjectKwargs {
            version: Option<String>,
            // meson_version: Option<String>,
            // license: Vec<String>,
            // subproject_dir: String,
        }

        let mut project_args = ProjectKwargs {
            version: None,
            // meson_version: None,
            // license: Vec::new(),
            // subproject_dir: String::new(),
        };

        assert!(
            args.len() >= 2,
            "project function requires at least 'project name' and 'language'"
        );
        assert!(matches!(args[0], ElementaryTypes::Str(_)));
        assert!(
            matches!(args[1], ElementaryTypes::Str(_))
                || matches!(args[1], ElementaryTypes::List(_))
        );

        let project_name = if let ElementaryTypes::Str(project_name) = &args[0] {
            project_name.clone()
        } else {
            String::new()
        };

        let project_langs = match &args[1] {
            ElementaryTypes::List(langs_list) => {
                let mut langs = Vec::new();

                for l in langs_list {
                    if let ElementaryTypes::Str(s) = l {
                        langs.push(s.to_owned());
                    }
                }

                langs
            }
            ElementaryTypes::Str(s) => vec![s.to_owned()],
            _ => panic!("Unknown arguments"),
        };

        assert!(
            !project_name.contains(':'),
            "Project name can't contain ':'"
        );

        // TODO process meson_options.txt

        info!("Project Name: {}", &project_name);
        info!("Project version: {:?}", project_args.version);

        self.build.project_name = project_name;

        if let ElementaryTypes::Str(v) = &kwargs["version"] {
            project_args.version = Some(v.to_string());
        }

        self.add_languages(&project_langs, true, MachineChoice::Host);
        // self.add_languages(&project_langs, false, MachineChoice::Build);
    }

    fn func_executable(
        &mut self,
        node: &Node,
        args: Vec<ElementaryTypes>,
        kwargs: HashMap<String, ElementaryTypes>,
    ) -> Option<Object> {
        let mut build_target = TargetType::BuildTarget(BuildTarget::new());
        self.build_target(node, args, kwargs, &mut build_target)
    }

    fn func_library(&mut self) -> Option<Object> {
        None
    }
}

mod tests {
    use super::Interpreter;
    use crate::{build::Build, environment::Environment, parser::parser::Parser};
    use std::path::Path;

    fn run_interpreter(inter: &mut Interpreter) {
        inter.sanity_check_ast();
        inter.parse_project();
        inter.redetect_machines();
        inter.run();
    }

    #[test]
    fn executable_test() {
        let code = r"
            project('simple', ['cpp'], version: '0.1')

            e = executable('simple_exe', 'main.cpp')
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

        assert!(!inter.build.targets.is_empty());
        assert!(inter.build.targets.contains_key("simple_exe"));

        assert!(!inter.variables.is_empty());
    }
}
