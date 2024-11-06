use std::{fs, path::PathBuf};

use anyhow::Result;
use reson::interpreter::Interpreter;

#[test]
fn interpret_cases() -> Result<()> {
    struct Test<'a> {
        file_contents: &'a str,
        compile: bool,
    }

    let tests = vec![
        Test {
            file_contents: "",
            compile: true,
        },
        Test {
            file_contents: "project()",
            compile: false,
        },
    ];

    for test in tests {
        let test_dir = tempfile::tempdir()?;

        let meson = test_dir.path().join("meson.build");
        fs::write(meson, test.file_contents)?;

        let mut interpreter = Interpreter::new(test_dir.path(), &PathBuf::new());
        assert_eq!(interpreter.interpret().is_ok(), test.compile);
    }

    Ok(())
}

#[test]
fn missing_build() {
    let mut interpreter = Interpreter::new(&PathBuf::from("missing/path"), &PathBuf::new());
    assert!(interpreter.interpret().is_err());
}

#[test]
fn compile_test() -> Result<()> {
    let current_file = PathBuf::from(file!());
    let current_dir = current_file.parent().unwrap().join("simple");
    println!("Current directory: {:?}", current_dir);

    let mut interpreter = Interpreter::new(&current_dir, &current_dir.join("build"));
    interpreter.interpret()?;

    Ok(())
}
