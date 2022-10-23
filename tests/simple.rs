use assert_cmd::prelude::*;
use std::{fs, process::Command};

#[test]
fn simple_test() -> Result<(), Box<dyn std::error::Error>> {
    let cwd = fs::canonicalize("tests/build_scripts/simple")
        .expect("Failed to get working directory for test");

    // ==== Setup ==== //
    println!("======= Setup ===== ");
    let mut cmd = Command::cargo_bin("reson")?;

    cmd.arg("setup").arg("build").current_dir(&cwd);
    // cmd.assert().success();

    let output = &cmd.output().expect("Failed to get output of command");
    let stdout = &output.stdout;
    let stderr = &output.stderr;
    println!("--- stdout\n{:?}", std::str::from_utf8(stdout).unwrap());
    println!("--- stderr\n{:?}", std::str::from_utf8(stderr).unwrap());

    let build_dir = cwd.join("build");
    assert!(build_dir.exists());

    // ==== Build ==== //
    println!("======= Building ===== ");
    let mut cmd = Command::new("ninja");
    cmd.current_dir(&build_dir);
    cmd.current_dir(&build_dir);
    let output = &cmd.output().expect("Failed to get output of command");
    let stdout = &output.stdout;
    let stderr = &output.stderr;
    println!("--- stdout\n{:?}", std::str::from_utf8(stdout).unwrap());
    println!("--- stderr\n{:?}", std::str::from_utf8(stderr).unwrap());

    // ==== Run ===== //
    println!("======= Running ===== ");
    let mut cmd = Command::new("./simple");
    cmd.current_dir(&build_dir);
    let output = &cmd.output().expect("Failed to get output of command");
    let stdout = &output.stdout;
    let stderr = &output.stderr;
    println!("--- stdout\n{:?}", std::str::from_utf8(stdout).unwrap());
    println!("--- stderr\n{:?}", std::str::from_utf8(stderr).unwrap());

    // Cleanup
    println!("====== Cleanup ======");
    fs::remove_dir_all(build_dir).expect("Failed to remove build directory");

    Ok(())
}
