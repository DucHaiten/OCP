use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn v100_release_portable_workflow_contract() {
    let workflow_path = repo_root()
        .join(".github")
        .join("workflows")
        .join("w100-release-portable-assets.yml");
    assert!(workflow_path.exists(), "missing {}", workflow_path.display());

    let raw = fs::read_to_string(&workflow_path)
        .unwrap_or_else(|_| panic!("read {}", workflow_path.display()));

    for needle in [
        "workflow_dispatch:",
        "runs-on: ubuntu-latest",
        "runs-on: macos-14",
        "cargo build --release -p ocl-cli -p ocl-lsp -p ocl-dap --target x86_64-unknown-linux-musl",
        "cargo build --release -p ocl-cli -p ocl-lsp -p ocl-dap --target aarch64-apple-darwin",
        "ocl-v1.0.0-linux-x64.tar.gz",
        "ocl-v1.0.0-macos-arm64.tar.gz",
        "w100-portable-linux-x64",
        "w100-portable-macos-arm64",
    ] {
        assert!(
            raw.contains(needle),
            "workflow must contain `{needle}`"
        );
    }
}
