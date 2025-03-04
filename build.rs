use std::{env, process::Command};

fn main() {
    println!("cargo::rerun-if-changed=src/pages/");
    let mut out_dir = env::var("OUT_DIR").unwrap();
    out_dir.push_str("/pages/");

    if !Command::new("cp")
        .args(["-r", "src/pages/", &out_dir])
        .status()
        .unwrap()
        .success()
    {
        panic!("cp failed");
    }

    if !Command::new("tsc")
        .arg("--outDir")
        .arg(&out_dir)
        .status()
        .unwrap()
        .success()
    {
        panic!("tsc failed")
    }
}
