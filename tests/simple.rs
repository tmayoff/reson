use assert_cmd::prelude::*;
use std::{fs, process::Command};

#[test]
fn simple_test() -> Result<(), Box<dyn std::error::Error>> {
    let cwd = fs::canonicalize("tests/build_scripts/simple")
        .expect("Failed to get working directory for test");

    let mut cmd = Command::cargo_bin("reson")?;

    cmd.arg("setup").arg("build").current_dir(&cwd);
    cmd.assert().success();

    let stdout = &cmd.output()?.stdout;
    let stderr = cmd.output()?.stderr;
    println!("--- stdout\n{}", std::str::from_utf8(stdout).unwrap());
    println!("--- stderr\n{}", std::str::from_utf8(&stderr).unwrap());

    let mut build_dir = cwd;
    build_dir.push("build");
    assert!(build_dir.exists());

    // Cleanup
    // assert!(fs::remove_dir(build_dir).is_ok());

    Ok(())
}
