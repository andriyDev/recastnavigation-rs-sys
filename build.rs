use std::{env, path::PathBuf};

use cmake::Config;

fn main() {
    println!("cargo:rerun-if-env-changed=PROFILE");

    let lib_destination = find_recast().unwrap_or_else(|| build_recast());

    println!(
        "cargo:rustc-link-search=native={}",
        lib_destination.display()
    );

    for lib in lib_names() {
        println!("cargo:rustc-link-lib=static={}", lib);
    }
}

fn lib_names() -> Vec<String> {
    let root_names = vec!["Recast", "Detour", "DetourCrowd", "DetourTileCache"];
    if env::var("PROFILE").unwrap() == "debug" {
        root_names
            .iter()
            .map(|root| root.to_string() + "-d")
            .collect()
    } else {
        root_names.iter().map(|root| root.to_string()).collect()
    }
}

fn find_recast() -> Option<PathBuf> {
    None
}

fn build_recast() -> PathBuf {
    println!("cargo:rerun-if-changed=recastnavigation");
    let mut lib_builder = Config::new("recastnavigation");
    lib_builder
        .define("RECASTNAVIGATION_DEMO", "OFF")
        .define("RECASTNAVIGATION_EXAMPLES", "OFF")
        .define("RECASTNAVIGATION_TESTS", "OFF");
    let lib_destination = lib_builder.build();
    lib_destination.join("lib")
}
