use std::env;
use std::process::Command;

fn main() {
    let force_static = env::var("HADES_STATIC_LLVM").is_ok();

    let is_release = env::var("PROFILE").map(|p| p == "release").unwrap_or(false);

    if !is_release && !force_static {
        return;
    }

    if env::var("HADES_DYNAMIC_LLVM").is_ok() {
        println!("cargo:warning=Using dynamic LLVM linking");
        return;
    }

    let llvm_config = env::var("LLVM_CONFIG").unwrap_or_else(|_| "llvm-config".to_string());

    println!("cargo:rerun-if-env-changed=LLVM_CONFIG");
    println!("cargo:rerun-if-env-changed=HADES_DYNAMIC_LLVM");

    let version = Command::new(&llvm_config)
        .arg("--version")
        .output()
        .expect("Failed to run llvm-config --version");

    let version_str = String::from_utf8_lossy(&version.stdout);
    println!(
        "cargo:warning=Building with LLVM version: {}",
        version_str.trim()
    );

    let libs = Command::new(&llvm_config)
        .args(["--link-static", "--libs", "--system-libs"])
        .output()
        .expect("Failed to run llvm-config --link-static --libs --system-libs");

    if !libs.status.success() {
        panic!(
            "llvm-config failed: {}",
            String::from_utf8_lossy(&libs.stderr)
        );
    }

    for lib in String::from_utf8_lossy(&libs.stdout).split_whitespace() {
        if let Some(name) = lib.strip_prefix("-l") {
            if name.starts_with("LLVM") {
                println!("cargo:rustc-link-lib=static={}", name);
            } else {
                println!("cargo:rustc-link-lib={}", name);
            }
        }
    }

    let ldflags = Command::new(&llvm_config)
        .args(["--link-static", "--ldflags"])
        .output()
        .expect("Failed to run llvm-config --ldflags");

    for flag in String::from_utf8_lossy(&ldflags.stdout).split_whitespace() {
        if let Some(path) = flag.strip_prefix("-L") {
            println!("cargo:rustc-link-search=native={}", path);
        }
    }

    let target = env::var("TARGET").unwrap();

    match target {
        t if t.contains("linux") => {
            println!("cargo:rustc-link-lib=static=stdc++");
            println!("cargo:rustc-link-lib=gcc_s");
        }
        t if t.contains("apple") => {
            println!("cargo:rustc-link-lib=c++");
        }
        t if t.contains("windows-gnu") => {
            println!("cargo:rustc-link-lib=static=stdc++");
            println!("cargo:rustc-link-lib=static=gcc");
        }
        t if t.starts_with("riscv") || t.starts_with("arm") || t.starts_with("mips") => {
            println!("cargo:rustc-link-lib=atomic");
        }
        _ => panic!("Unsupported target: {}", target),
    }
}
