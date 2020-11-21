use std::path::PathBuf;

fn main() {
    println!(
        "cargo:include={}",
        PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap())
            .join("include")
            .display()
    );
}
