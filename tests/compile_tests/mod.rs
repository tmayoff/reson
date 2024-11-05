use std::path::PathBuf;

use anyhow::Result;
use reson::interpreter::Interpreter;

#[test]
fn compile_test() -> Result<()> {
    let current_file = PathBuf::from(file!());
    let current_dir = current_file.parent().unwrap().join("simple");
    println!("Current directory: {:?}", current_dir);

    let mut interpreter = Interpreter::new(&current_dir, &current_dir.join("build"));
    interpreter.interpret()?;

    Ok(())
}
