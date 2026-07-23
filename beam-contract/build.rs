fn main() {
    // Shrink the WASM shadow stack (default 1 MiB) to cut the linear memory
    // Soroban charges for every contract invocation. Wasm-only: native test
    // binaries must not receive this linker flag.
    let target = std::env::var("TARGET").unwrap_or_default();
    if target.starts_with("wasm32") {
        println!("cargo:rustc-link-arg=-zstack-size=16384");
    }
}
