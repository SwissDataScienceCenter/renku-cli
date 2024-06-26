use vergen::EmitBuilder;

pub fn main() {
    EmitBuilder::builder()
        .all_build()
        .all_git()
        .all_cargo()
        .all_rustc()
        .emit()
        .unwrap();
}
