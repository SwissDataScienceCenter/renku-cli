use vergen_gitcl::{Build, Cargo, Emitter, Gitcl, Rustc};

pub fn main() {
    if let Ok(val) = std::env::var("RNK_RELEASE_VERSION") {
        println!("cargo:rustc-env=CARGO_PKG_VERSION={}", val);
    }
    println!("cargo:rerun-if-env-changed=RNK_RELEASE_VERSION");
    Emitter::default()
        .add_instructions(&Build::all_build())
        .unwrap()
        .add_instructions(&Cargo::all_cargo())
        .unwrap()
        .add_instructions(&Gitcl::all_git())
        .unwrap()
        .add_instructions(&Rustc::all_rustc())
        .unwrap()
        .emit()
        .unwrap();
}
