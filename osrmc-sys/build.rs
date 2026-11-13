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
        .include("src")
        .file("src/libosrmc/libosrmc/osrmc.cc")
        .file("src/osrmc_ext.cc");

    let libosrm = pkg_config::Config::new()
        .probe("libosrm")
        .expect("Could not call pkg-config for libosrm");
    println!("pkgconfig: {:?}", libosrm);
    // The raw-result additions need the patched OSRM (see src/osrmc_ext.h); against a stock
    // build they compile to stubs that report NotImplemented.
    let patched = libosrm.include_paths.iter().any(|path| {
        path.join("osrm/snap_cache_counters.hpp").exists()
            || path.join("snap_cache_counters.hpp").exists()
    });
    println!("cargo:rerun-if-env-changed=OSRMC_EXT_REQUIRE_RAW");
    let required = env::var_os("OSRMC_EXT_REQUIRE_RAW").is_some_and(|value| value == "1");
    if !patched && required {
        panic!(
            "OSRMC_EXT_REQUIRE_RAW=1 but the libosrm found by pkg-config has no \
             TableRaw/TableRows/RouteRaw (osrm/snap_cache_counters.hpp not in {:?})",
            libosrm.include_paths
        );
    }
    if !patched {
        println!("cargo:warning=OSRM without TableRaw/TableRows/RouteRaw: raw results unavailable");
        build.define("OSRMC_EXT_STOCK_OSRM", None);
    }
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
    for file in ["src/wrapper.h", "src/osrmc_ext.h", "src/osrmc_ext.cc"] {
        println!("cargo:rerun-if-changed={}", file);
    }
    compile_libosrmc();
    generate_libosrmc_bindings();

    println!("cargo:rustc-link-lib=boost_system");
    println!("cargo:rustc-link-lib=boost_filesystem");
    println!("cargo:rustc-link-lib=boost_iostreams");
    // The homebrew `osrm-backend` package specifies tbb@2020 as a dependency which is keg only and
    // therefore not symlinked to a directory where the linker can find it. `rustc-link-search` is a
    // hack which tells rustc to add the TBB location to the linker search path. Note that the
    // DYLD_LIBRARY_PATH environment variable can no longer be used on MacOS due to System Integrity
    // Protection. `/usr/local` is the Homebrew install path on Intel and `/opt/homebrew` is for
    // Apple Silicon.
    println!("cargo:rustc-link-search=/usr/local/opt/tbb@2020/lib");
    println!("cargo:rustc-link-search=/opt/homebrew/opt/tbb@2020/lib");
    println!("cargo:rustc-link-search=/opt/homebrew/opt/boost/lib");
    println!("cargo:rustc-link-lib=tbb");

    // Boost library names differ on macOS.
    if env::var("TARGET").unwrap().contains("apple") {
        println!("cargo:rustc-link-lib=boost_thread-mt");
    } else {
        println!("cargo:rustc-link-lib=boost_thread");
    }

}
