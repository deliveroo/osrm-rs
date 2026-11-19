use std::env;
use std::path::PathBuf;

/// Whether the `raw-results` feature is on.
fn raw_results() -> bool {
    env::var_os("CARGO_FEATURE_RAW_RESULTS").is_some()
}

/// `OSRMC_EXT_REQUIRE_RAW=1` makes the build fail unless the raw results are available.
fn require_raw() -> bool {
    env::var_os("OSRMC_EXT_REQUIRE_RAW").is_some_and(|value| value == "1")
}

/// Compiles src/osrmc_ext.cc, the C side of the `raw-results` feature.
fn add_raw_results(build: &mut cc::Build, libosrm: &pkg_config::Library) {
    build.include("src").file("src/osrmc_ext.cc");
    // The raw-result additions need the patched OSRM (see src/osrmc_ext.h); against a stock
    // build they compile to stubs that report NotImplemented.
    let patched = libosrm.include_paths.iter().any(|path| {
        path.join("osrm/snap_cache_counters.hpp").exists()
            || path.join("snap_cache_counters.hpp").exists()
    });
    if !patched && require_raw() {
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
}

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
    println!("cargo:rerun-if-env-changed=OSRMC_EXT_REQUIRE_RAW");
    if raw_results() {
        add_raw_results(&mut build, &libosrm);
    } else if require_raw() {
        panic!("OSRMC_EXT_REQUIRE_RAW=1 needs the raw-results feature");
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
    let mut builder = bindgen::Builder::default().header("src/wrapper.h");
    if raw_results() {
        builder = builder.header("src/osrmc_ext.h");
    }
    let bindings = builder
        .generate()
        .expect("Unable to generate bindings for libosrmc");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings for libosrmc");
}

fn main() {
    if raw_results() {
        for file in ["src/wrapper.h", "src/osrmc_ext.h", "src/osrmc_ext.cc"] {
            println!("cargo:rerun-if-changed={}", file);
        }
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
