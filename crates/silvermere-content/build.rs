//! Embeds the repository's `content/` in the crate: every `.toml` file, so a binary installs
//! the content of the commit it was built from.

use std::fmt::Write as _;
use std::path::PathBuf;

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let dir = manifest.join("../../content");
    println!("cargo:rerun-if-changed={}", dir.display());
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .map(|entries| {
            entries
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|e| e == "toml"))
                .collect()
        })
        .unwrap_or_default();
    files.sort();
    let mut out = String::from("pub static FILES: &[(&str, &str)] = &[\n");
    for f in &files {
        println!("cargo:rerun-if-changed={}", f.display());
        let name = f
            .file_name()
            .expect("a file name")
            .to_string_lossy()
            .to_string();
        let abs = f
            .canonicalize()
            .expect("the content file")
            .display()
            .to_string();
        writeln!(out, "    ({name:?}, include_str!({abs:?})),").unwrap();
    }
    out.push_str("];\n");
    let dest = PathBuf::from(std::env::var("OUT_DIR").expect("out dir")).join("embedded.rs");
    std::fs::write(dest, out).expect("writing embedded.rs");
}
