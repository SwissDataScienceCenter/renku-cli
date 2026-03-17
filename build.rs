use vergen::EmitBuilder;

pub fn main() {
    if let Ok(val) = std::env::var("RNK_RELEASE_VERSION") {
        println!("cargo:rustc-env=CARGO_PKG_VERSION={}", val);
    }
    println!("cargo:rerun-if-env-changed=RNK_RELEASE_VERSION");
    EmitBuilder::builder()
        .all_build()
        .all_git()
        .all_cargo()
        .all_rustc()
        .emit()
        .unwrap();
}
