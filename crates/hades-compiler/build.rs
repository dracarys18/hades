use std::process::Command;

fn main() {
    let out = std::env::var("OUT_DIR").unwrap();
    let archive = format!("{out}/libhades_runtime.a");
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into());

    let status = Command::new(rustc)
        .args([
            "--edition=2021",
            "--crate-type=staticlib",
            "--crate-name=hades_runtime",
            "-C",
            "panic=abort",
            "-O",
            "../hades-runtime/src/lib.rs",
            "-o",
            &archive,
        ])
        .status()
        .expect("failed to invoke rustc for runtime");
    assert!(status.success(), "runtime compilation failed");

    println!("cargo:rerun-if-changed=../hades-runtime/src");
    println!("cargo:rerun-if-changed=build.rs");
}
