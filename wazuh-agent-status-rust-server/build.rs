use std::path::Path;

fn main() {
    let script_path = Path::new("../scripts/windows/adorsys-update.ps1");
    if !script_path.exists() {
        panic!(
            "Build failed: Required file 'scripts/windows/adorsys-update.ps1' not found.\n\
             If the directory structure was reorganized, please update the path in \
             `wazuh-agent-status-rust-server/src/manager.rs` and `wazuh-agent-status-rust-server/build.rs`."
        );
    }

    // Tell Cargo to re-run this script if the PS1 file changes
    println!("cargo:rerun-if-changed=../scripts/windows/adorsys-update.ps1");
}
