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

fn is_windows() -> bool {
    env::var("CARGO_CFG_WINDOWS").is_ok()
}

fn is_debug() -> bool {
    env::var("DEBUG").unwrap() == "true"
}

fn lib_names() -> Vec<String> {
    let root_names = vec!["Recast", "Detour", "DetourCrowd", "DetourTileCache"];
    if is_windows() && is_debug() {
        root_names
            .iter()
            .map(|root| root.to_string() + "-d")
            .collect()
    } else {
        root_names.iter().map(|root| root.to_string()).collect()
    }
}

fn lib_name_to_file_name(lib_name: &str) -> String {
    if is_windows() {
        format!("{}.lib", lib_name)
    } else {
        format!("lib{}.a", lib_name)
    }
}

fn find_recast() -> Option<PathBuf> {
    let lib = match pkg_config::Config::new().probe("recastnavigation") {
        Ok(value) => value,
        Err(error) => {
            println!("pkg_config failed to find RecastNavigation: {}", error);
            return None;
        }
    };

    if lib.link_paths.len() != 1 {
        println!(
            "cargo:warning=Expected 1 link path from recastnavigation, got {:?}",
            lib.link_paths
        );
        return None;
    }

    let lib_dir = &lib.link_paths[0];

    let lib_names = lib_names();

    let check_libs = lib_names
        .iter()
        .map(|lib_name| {
            lib_dir
                .join(lib_name_to_file_name(lib_name))
                .as_path()
                .exists()
        })
        .collect::<Vec<bool>>();
    if check_libs.iter().all(|b| *b) {
        Some(lib_dir.clone())
    } else {
        let missing_libs = lib_names
            .iter()
            .zip(check_libs)
            .filter_map(|(lib_name, present)| {
                if present {
                    None
                } else {
                    Some(lib_name.as_str())
                }
            })
            .collect::<Vec<&str>>();
        println!("cargo:warning=Found recastnavigation using pkgconfig at {}, but not all libraries were present. Missing libs: {:?}", lib_dir.display(), missing_libs);

        None
    }
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
