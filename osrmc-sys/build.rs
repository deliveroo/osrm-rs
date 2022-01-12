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
    println!("cargo:rustc-link-lib=boost_filesystem");
    println!("cargo:rustc-link-lib=boost_iostreams");
    // The homebrew `osrm-backend` package specifies tbb@2020 as a dependency which is keg only and
    // therefore not symlinked to a directory where the linker can find it. `rustc-link-search` is a
    // hack which tells rustc to add the TBB location to the linker search path. Note that the
    // DYLD_LIBRARY_PATH environment variable can no longer be used on MacOS due to System Integrity
    // Protection.
    println!("cargo:rustc-link-search=/usr/local/opt/tbb@2020/lib");
    println!("cargo:rustc-link-lib=tbb");

    // Boost library names differ on macOS.
    if env::var("TARGET").unwrap().contains("apple") {
        println!("cargo:rustc-link-lib=boost_thread-mt");
    } else {
        println!("cargo:rustc-link-lib=boost_thread");
    }

}
