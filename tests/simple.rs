use assert_cmd::{assert, prelude::*};
use file_diff::diff_files;
use std::{
    fs::{self, File},
    process::Command,
};

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

    // let mut generated = File::open("tests/build_scripts/simple/build/build.ninja")
    //     .expect("Couldn't open generated build file");
    // let mut real =
    //     File::open("tests/build_scripts/simple/build.ninja").expect("Failed to open truth file");

    // assert!(diff_files(&mut generated, &mut real));

    // Cleanup
    let rm_build_dir = fs::remove_dir_all(build_dir);
    assert!(rm_build_dir.is_ok(), "{:?}", rm_build_dir.err());

    Ok(())
}
