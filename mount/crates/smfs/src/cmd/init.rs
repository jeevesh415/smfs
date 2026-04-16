//! `smfs init` — install the grep shell wrapper.

use anyhow::Result;
use clap::Args as ClapArgs;

#[derive(ClapArgs, Debug)]
pub struct Args {}

const SHELL_WRAPPER: &str = r#"
# supermemoryfs grep wrapper — semantic search inside mounted containers
grep() {
    for arg in "$@"; do
        case "$arg" in
            -*) command grep "$@"; return ;;
        esac
    done
    _smfs_dir="$PWD"
    while [ "$_smfs_dir" != "/" ]; do
        if [ -f "$_smfs_dir/.smfs" ]; then
            smfs grep "$@"
            return
        fi
        _smfs_dir="$(dirname "$_smfs_dir")"
    done
    command grep "$@"
}
"#;

const MARKER: &str = "supermemoryfs grep wrapper";

/// Check if the grep wrapper is installed in ~/.zshrc. If not, install it.
/// Returns true if newly installed, false if already present.
pub fn ensure_grep_wrapper_installed() -> Result<bool> {
    let home = std::env::var("HOME").map(std::path::PathBuf::from)?;
    let zshrc = home.join(".zshrc");

    if let Ok(content) = std::fs::read_to_string(&zshrc) {
        if content.contains(MARKER) {
            return Ok(false);
        }
    }

    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&zshrc)?;
    file.write_all(SHELL_WRAPPER.as_bytes())?;

    Ok(true)
}

pub async fn run(_args: Args) -> Result<()> {
    if ensure_grep_wrapper_installed()? {
        eprintln!("semantic grep installed. run: source ~/.zshrc");
    } else {
        eprintln!("semantic grep already installed.");
    }
    Ok(())
}
