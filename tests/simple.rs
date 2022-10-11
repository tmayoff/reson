use std::{process::Command, fs};
use assert_cmd::prelude::*;

#[test]
fn simple_test() -> Result<(), Box<dyn std::error::Error>>{
    let mut cmd = Command::cargo_bin("reson")?;

    cmd.arg("setup").arg("-C").arg(" build").current_dir(fs::canonicalize("tests/build_scripts/simple")?);
    cmd.assert().success();

    Ok(())
}