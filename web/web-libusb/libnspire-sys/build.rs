use globwalk::DirEntry;

fn main() {
    let files = globwalk::GlobWalkerBuilder::from_patterns("libnspire/src", &["*.{c,cpp}"])
        .build()
        .unwrap()
        .into_iter()
        .filter_map(Result::ok)
        .map(DirEntry::into_path)
        .inspect(|path| println!("cargo:rerun-if-changed={}", path.display()));
    let mut cfg = cc::Build::new();
    if let Some(include) = std::env::var_os("DEP_USB_1.0_INCLUDE") {
        cfg.include(include);
    }
    cfg.files(files)
        .include("libnspire/src")
        .include("libnspire/src/services")
        .include("libnspire/src/api")
        .compile("nspire");

    // let bindings = bindgen::Builder::default()
    //     // The input header we would like to generate
    //     // bindings for.
    //     .header("libnspire/src/api/nspire.h")
    //     // Tell cargo to invalidate the built crate whenever any of the
    //     // included header files changed.
    //     .parse_callbacks(Box::new(bindgen::CargoCallbacks))
    //     .whitelist_function("(nspire.*|free)")
    //     .whitelist_type("nspire.*")
    //     .whitelist_var("nspire.*")
    //     .whitelist_type("NSPIRE.*")
    //     .whitelist_var("NSPIRE.*")
    //     .rustified_enum("nspire_keys")
    //     .size_t_is_usize(true)
    //     // Finish the builder and generate the bindings.
    //     .generate()
    //     // Unwrap the Result and panic on failure.
    //     .expect("Unable to generate bindings");
    //
    // // Write the bindings to the $OUT_DIR/bindings.rs file.
    // bindings
    //     .write_to_file("src/bindings.rs")
    //     .expect("Couldn't write bindings!");
}
