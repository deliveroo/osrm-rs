use std::env;
use std::path::PathBuf;

fn compile_libosrmc() {
    env::set_var(
        "CXXFLAGS",
        env::var("CXXFLAGS").unwrap_or_default() + " -std=c++14",
    );

    let mut build = cc::Build::new();
    build
        .cpp(true)
        .include("src/libosrmc/libosrmc")
        .file("src/libosrmc/libosrmc/osrmc.cc");

    let libosrm = pkg_config::Config::new()
        .probe("libosrm")
        .expect("Could not call pkg-config for libosrm");
    println!("pkgconfig: {:?}", libosrm);
    for include in libosrm.include_paths {
        build.include(include);
    }
    for link_path in libosrm.link_paths {
        if let Some(path) = link_path.to_str() {
            println!("cargo:rustc-link-search=native={}", path);
        }
    }

    build.compile("libosrmc");
}

fn generate_libosrmc_bindings() {
    let bindings = bindgen::Builder::default()
        .header("src/wrapper.h")
        .generate()
        .expect("Unable to generate bindings for libosrmc");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings for libosrmc");
}

fn main() {
    compile_libosrmc();
    generate_libosrmc_bindings();

    println!("cargo:rustc-link-lib=boost_system");
    println!("cargo:rustc-link-lib=boost_thread-mt");
    // These might be needed for other features in OSRM:
    println!("cargo:rustc-link-lib=boost_filesystem");
    println!("cargo:rustc-link-lib=boost_iostreams-mt");
    println!("cargo:rustc-link-lib=tbb");
}
