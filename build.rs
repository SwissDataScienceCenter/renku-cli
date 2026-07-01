use vergen_gitcl::{BuildBuilder, CargoBuilder, Emitter, GitclBuilder, RustcBuilder};

pub fn main() {
    if let Ok(val) = std::env::var("RNK_RELEASE_VERSION") {
        println!("cargo:rustc-env=CARGO_PKG_VERSION={}", val);
    }
    println!("cargo:rerun-if-env-changed=RNK_RELEASE_VERSION");
    Emitter::default()
        .add_instructions(&BuildBuilder::all_build().unwrap())
        .unwrap()
        .add_instructions(&CargoBuilder::all_cargo().unwrap())
        .unwrap()
        .add_instructions(&GitclBuilder::all_git().unwrap())
        .unwrap()
        .add_instructions(&RustcBuilder::all_rustc().unwrap())
        .unwrap()
        .emit()
        .unwrap();
}
