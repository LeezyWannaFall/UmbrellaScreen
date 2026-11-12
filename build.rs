fn main() {
    let local = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join(".native/usr/lib");
    if local.exists() {
        println!("cargo:rustc-link-search=native={}", local.display());
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", local.display());
    }
    println!("cargo:rustc-link-lib=gtk4-layer-shell");
}
