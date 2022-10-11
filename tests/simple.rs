use std::{process::Command, fs};
use assert_cmd::prelude::*;

#[test]
fn simple_test() -> Result<(), Box<dyn std::error::Error>>{
    let mut cmd = Command::cargo_bin("reson")?;

    cmd.arg("setup").arg("-C").arg("build").current_dir(fs::canonicalize("tests/build_scripts/simple")?);
    cmd.assert().success();

    let stdout = &cmd.output()?.stdout;
    let stderr = cmd.output()?.stderr;
    println!("--- stdout\n{}", std::str::from_utf8(stdout).unwrap());
    println!("--- stderr\n{}", std::str::from_utf8(&stderr).unwrap());

    Ok(())
}