//! Build script: compile behavior tree to Rust validation code.
//!
//! Set DSHIP_VALIDATION_TREE to override the default validation-tree.json path.

use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR");
    let output_path = Path::new(&out_dir).join("validation.rs");

    let tree_path = env::var("DSHIP_VALIDATION_TREE")
        .or_else(|_| env::var("DSHIP_VALIDATION_TREE_PATH"))
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            Path::new(&manifest_dir)
                .join("validation-tree.json")
                .to_string_lossy()
                .into_owned()
        });

    println!("cargo:rerun-if-changed={}", tree_path);

    let workspace_root = Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root");

    let status = Command::new("cargo")
        .current_dir(workspace_root)
        .args([
            "run",
            "-p",
            "dship-validation-bt",
            "--bin",
            "dship-bt-compile",
            "--",
            &tree_path,
            "--output",
            output_path.to_str().expect("output path"),
        ])
        .status()
        .expect("failed to run dship-bt-compile");

    if !status.success() {
        panic!("dship-bt-compile failed");
    }
}
