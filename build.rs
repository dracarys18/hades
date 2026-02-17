use std::env;
use std::env;
use std::process::Command;

fn main() {
    // Only do static linking for release builds or when explicitly requested
    let profile = env::var("PROFILE").unwrap_or_default();
    let force_static = env::var("HADES_STATIC_LLVM").is_ok();

    if profile != "release" && !force_static {
        // Skip static linking for dev/test builds
        return;
    }

    // Allow disabling static linking even in release mode
    if env::var("HADES_DYNAMIC_LLVM").is_ok() {
        println!("cargo:warning=Using dynamic LLVM linking (development mode)");
        return;
    }

    let llvm_config = env::var("LLVM_CONFIG").unwrap_or_else(|_| "llvm-config".to_string());

    println!("cargo:rerun-if-env-changed=LLVM_CONFIG");
    println!("cargo:rerun-if-env-changed=HADES_DYNAMIC_LLVM");

    // Get LLVM version for verification
    let version = Command::new(&llvm_config)
        .arg("--version")
        .output()
        .expect("Failed to run llvm-config --version");

    let version_str = String::from_utf8_lossy(&version.stdout);
    println!(
        "cargo:warning=Building with LLVM version: {}",
        version_str.trim()
    );

    // Get static LLVM libraries
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

    // Parse and emit library link directives
    for lib in String::from_utf8_lossy(&libs.stdout).split_whitespace() {
        if let Some(name) = lib.strip_prefix("-l") {
            // Static linking for LLVM libraries
            if name.starts_with("LLVM") {
                println!("cargo:rustc-link-lib=static={}", name);
            } else {
                // System libraries stay dynamic
                println!("cargo:rustc-link-lib={}", name);
            }
        }
    }

    // Get linker search paths
    let ldflags = Command::new(&llvm_config)
        .args(["--link-static", "--ldflags"])
        .output()
        .expect("Failed to run llvm-config --ldflags");

    for flag in String::from_utf8_lossy(&ldflags.stdout).split_whitespace() {
        if let Some(path) = flag.strip_prefix("-L") {
            println!("cargo:rustc-link-search=native={}", path);
        }
    }

    // Platform-specific C++ standard library linking
    let target = env::var("TARGET").unwrap();

    if target.contains("linux") {
        // On Linux, link libstdc++ statically to avoid glibc version issues
        println!("cargo:rustc-link-lib=static=stdc++");
        println!("cargo:rustc-link-lib=gcc_s");
    } else if target.contains("apple") {
        // On macOS, use libc++ (comes with the system)
        println!("cargo:rustc-link-lib=c++");
    } else if target.contains("windows-msvc") {
        // On Windows MSVC, the static CRT handles C++ stdlib
        // The +crt-static flag in .cargo/config.toml handles this
    } else if target.contains("windows-gnu") {
        // On Windows GNU, link libstdc++ statically
        println!("cargo:rustc-link-lib=static=stdc++");
        println!("cargo:rustc-link-lib=static=gcc");
    }

    // Some architectures need libatomic
    if target.starts_with("riscv") || target.starts_with("arm") || target.starts_with("mips") {
        println!("cargo:rustc-link-lib=atomic");
    }
}
