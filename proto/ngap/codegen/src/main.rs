//! Compiles the NGAP ASN.1 modules of one TS 38.413 version into
//! `proto/ngap/src/generated.rs`.

mod procedures;

use std::path::{Path, PathBuf};

use rasn_compiler::OutputMode;
use rasn_compiler::prelude::*;

fn main() {
    let version = std::env::args().nth(1).expect("usage: ngap-codegen <version, e.g. 15.13.0>");
    let ngap = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf();
    let dir = ngap.join("asn1").join(&version);
    let mut sources: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .map(|entry| entry.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "asn"))
        .collect();
    sources.sort();
    let output = ngap.join("src").join("generated.rs");
    // Whole-module imports: the selective ones miss PrivateIE-ID in the contents module.
    let config = RasnConfig { default_wildcard_imports: true, ..Default::default() };
    let warnings = Compiler::<RasnBackend, _>::new_with_config(config)
        .add_asn_sources_by_path(sources.iter())
        .set_output_mode(OutputMode::SingleFile(output.clone()))
        .compile()
        .unwrap_or_else(|e| panic!("compilation failed: {e}"));
    for w in &warnings {
        eprintln!("warning: {w}");
    }
    println!("{}: {} warning(s)", output.display(), warnings.len());
    procedures::generate(&dir, &ngap.join("src").join("procedures.rs"), &version);
}
