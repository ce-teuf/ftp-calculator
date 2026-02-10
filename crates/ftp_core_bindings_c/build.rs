extern crate cbindgen;
use std::env;
use std::path::PathBuf;

fn main() {
    // Chemin de sortie pour le header C
    let crate_dir = match env::var("CARGO_MANIFEST_DIR") {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("Error getting CARGO_MANIFEST_DIR: {}", e);
            return;
        }
    };
    println!("cargo:warning=Crate directory: {}", crate_dir);

    let out_dir = PathBuf::from(&crate_dir)
        .join("..")
        .join("..")
        .join("excel-addin")
        .join("Interop");

    // Crée le dossier Interop uniquement s'il n'existe pas déjà
    if !out_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(&out_dir) {
            eprintln!("Error creating directory: {}", e);
            return;
        }
    }

    // Génère le header C
    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .generate()
        .map_or_else(
            |error| match error {
                cbindgen::Error::ParseSyntaxError { .. } => {}
                e => panic!("{:?}", e),
            },
            |bindings| {
                bindings.write_to_file(out_dir.join("bindings.h"));
            },
        )
}
