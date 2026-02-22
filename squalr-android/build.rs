use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let target_triple = env::var("TARGET").unwrap_or_default();
    if !target_triple.contains("android") {
        return;
    }

    let profile_name = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let out_directory = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR must be defined for build scripts."));
    let workspace_directory = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be defined."))
        .parent()
        .expect("squalr-android manifest must live under the workspace root.")
        .to_path_buf();
    let bundled_cli_path = out_directory.join("squalr-cli-bundle");
    let built_cli_path = workspace_directory
        .join("target")
        .join(&target_triple)
        .join(&profile_name)
        .join("squalr-cli");

    println!("cargo:rerun-if-changed={}", built_cli_path.display());

    if let Err(error) = fs::copy(&built_cli_path, &bundled_cli_path) {
        panic!(
            "Failed to bundle Android privileged CLI from {} to {}. \
Build `squalr-cli` first with `cargo ndk --target aarch64-linux-android build -p squalr-cli`.\n{}",
            built_cli_path.display(),
            bundled_cli_path.display(),
            error
        );
    }
}
