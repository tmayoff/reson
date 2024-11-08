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
        Test {
            file_contents: "project(hello_world)",
            compile: false,
        },
        Test {
            file_contents: "project('hello world')",
            compile: true,
        },
        Test {
            file_contents: "project('hello world')\nexecutable()",
            compile: false,
        },
        Test {
            file_contents: "project('hello world')\nexecutable('exe')",
            compile: true,
        },
        // Test {
        //     file_contents: r#"project('hello world')
        //     if get_option('buildtype') == 'debug'
        //         executable('exe')
        //     endif"#,
        //     compile: true,
        // },
    ];

    for test in tests {
        let test_dir = tempfile::tempdir()?;

        let meson = test_dir.path().join("meson.build");
        fs::write(meson, &test.file_contents)?;

        let mut interpreter = Interpreter::new(test_dir.path(), &PathBuf::new());
        let err = interpreter.interpret();

        match err {
            Ok(()) => {
                if !test.compile {
                    assert!(false, "Test: {}, should've failed", test.file_contents,);
                }
            }
            Err(e) => {
                if test.compile {
                    // should've succeeded
                    assert!(
                        false,
                        "Test: {}, should've succeeded but failed: {:?}",
                        test.file_contents, e
                    );
                }
            }
        }
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
