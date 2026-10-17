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
    let generated = std::fs::read_to_string(&output).unwrap();
    let (patched, lists) = patch_sequence_of_decoding(&generated);
    std::fs::write(&output, patched).unwrap();
    println!("{}: {} warning(s), {lists} list decoders patched", output.display(), warnings.len());
    procedures::generate(&dir, &ngap.join("src").join("procedures.rs"), &version);
}

/// Replaces the derived `Encode` and `Decode` of every size-constrained `SEQUENCE OF`
/// wrapper with `aper_sequence_of!` (see its doc in `src/lib.rs`):
///
/// ```text
/// #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
/// #[rasn(delegate, size("1..=12"))]
/// pub struct BroadcastPLMNList(pub SequenceOf<BroadcastPLMNItem>);
/// ```
fn patch_sequence_of_decoding(source: &str) -> (String, usize) {
    let lines: Vec<&str> = source.lines().collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut patched = 0;
    let mut i = 0;
    while i < lines.len() {
        // derive line, delegate line, then a declaration the formatter may wrap.
        let derive = lines[i];
        if i + 2 < lines.len() && derive.contains("derive(") && derive.contains(" Decode, Encode,") {
            let end = (i + 2..lines.len()).find(|&k| lines[k].trim_end().ends_with(';')).unwrap_or(i + 2);
            let def: String = lines[i + 2..=end].iter().map(|l| l.trim()).collect::<String>().replace(",)", ")");
            if let Some((name, item, lb, ub)) = sequence_of(lines[i + 1], &def) {
                let indent = &derive[..derive.len() - derive.trim_start().len()];
                out.push(derive.replacen(" Decode, Encode,", "", 1));
                out.extend(lines[i + 1..=end].iter().map(|l| l.to_string()));
                out.push(format!("{indent}aper_sequence_of!({name}, {item}, {lb}, {ub});"));
                patched += 1;
                i = end + 1;
                continue;
            }
        }
        out.push(derive.to_string());
        i += 1;
    }
    (out.join("\n") + "\n", patched)
}

/// `(name, item, lb, ub)` if `attrs` + `def` declare a size-constrained,
/// non-extensible `SEQUENCE OF` wrapper.
fn sequence_of(attrs: &str, def: &str) -> Option<(String, String, u64, u64)> {
    let rest = attrs.trim().strip_prefix("#[rasn(delegate, size(\"")?;
    let (size, after) = rest.split_once("\")")?;
    // Nothing else but an identifier: an extensible size would not parse as numbers below.
    if after != ")]" && !after.starts_with(", identifier = ") {
        return None;
    }
    let (lb, ub) = match size.split_once("..=") {
        Some((lb, ub)) => (lb.parse().ok()?, ub.parse().ok()?),
        None => (size.parse().ok()?, size.parse().ok()?),
    };
    let rest = def.trim().strip_prefix("pub struct ")?;
    let (name, inner) = rest.split_once("(pub SequenceOf<")?;
    let item = inner.strip_suffix(">);")?;
    Some((name.to_string(), item.to_string(), lb, ub))
}
