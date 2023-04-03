use std::{env, path::PathBuf};

use cmake::Config;

fn main() {
  println!("cargo:rerun-if-env-changed=PROFILE");
  println!("cargo:rerun-if-env-changed=RECAST_NO_VENDOR");
  println!("cargo:rerun-if-env-changed=RECAST_VENDOR");

  let lib_destination;
  if env::var("RECAST_NO_VENDOR").unwrap_or("false".into()) == "true" {
    lib_destination = find_recast().unwrap();
  } else if env::var("RECAST_VENDOR").unwrap_or("false".into()) == "true" {
    lib_destination = build_recast();
  } else {
    lib_destination = find_recast().unwrap_or_else(|| build_recast());
  }

  println!("cargo:rustc-link-search=native={}", lib_destination.display());

  for lib in lib_names() {
    println!("cargo:rustc-link-lib=static={}", lib);
  }

  build_and_link_inline_lib();
  generate_inline_bindings();

  generate_recast_bindings();
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
    root_names.iter().map(|root| root.to_string() + "-d").collect()
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
      lib_dir.join(lib_name_to_file_name(lib_name)).as_path().exists()
    })
    .collect::<Vec<bool>>();
  if check_libs.iter().all(|b| *b) {
    Some(lib_dir.clone())
  } else {
    let missing_libs = lib_names
      .iter()
      .zip(check_libs)
      .filter_map(
        |(lib_name, present)| {
          if present {
            None
          } else {
            Some(lib_name.as_str())
          }
        },
      )
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

fn generate_recast_bindings() {
  let bind_files: &[(&str, &str, &[&str])] = &[
    ("recastnavigation/Recast/Include/Recast.h", "recast.rs", &[]),
    ("recastnavigation/Detour/Include/DetourAlloc.h", "detour_Alloc.rs", &[]),
    (
      "recastnavigation/Detour/Include/DetourStatus.h",
      "detour_Status.rs",
      &[".*DetourAlloc\\.h"],
    ),
    (
      "recastnavigation/Detour/Include/DetourNavMesh.h",
      "detour_NavMesh.rs",
      &[".*DetourAlloc\\.h", ".*DetourStatus\\.h"],
    ),
    (
      "recastnavigation/Detour/Include/DetourNavMeshBuilder.h",
      "detour_NavMeshBuilder.rs",
      &[".*DetourAlloc\\.h"],
    ),
    (
      "recastnavigation/Detour/Include/DetourNavMeshQuery.h",
      "detour_NavMeshQuery.rs",
      &[".*DetourAlloc\\.h", ".*DetourNavMesh\\.h", ".*DetourStatus\\.h"],
    ),
  ];

  let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

  for &(bind_src, bind_dst, block_files) in bind_files {
    let mut bindings_builder = bindgen::Builder::default()
      .header(bind_src)
      .parse_callbacks(Box::new(bindgen::CargoCallbacks))
      .clang_args(["-x", "c++"].iter())
      .blocklist_file(".*stddef\\.h")
      .blocklist_type("max_align_t");

    for blocked_file in block_files {
      bindings_builder = bindings_builder.blocklist_file(blocked_file);
    }

    let bindings =
      bindings_builder.generate().expect("Unable to generate bindings.");

    bindings
      .write_to_file(out_path.join(bind_dst))
      .expect("Couldn't write bindings!");
  }
}

fn build_and_link_inline_lib() {
  println!("cargo:rerun-if-changed=inline_lib_src");

  cc::Build::new()
    .file("inline_lib_src/inline.cc")
    .include("recastnavigation/Recast/Include")
    .compile("recast_inline");

  println!("cargo:rustc-link-search=native={}", env::var("OUT_DIR").unwrap());
  println!("cargo:rustc-link-lib=static=recast_inline");
}

fn generate_inline_bindings() {
  let bindings = bindgen::Builder::default()
    .header("inline_lib_src/inline.h")
    .blocklist_type("rcContext")
    .parse_callbacks(Box::new(bindgen::CargoCallbacks))
    .clang_args(["-x", "c++"].iter())
    .generate()
    .expect("Unable to generate bindings.");

  let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
  bindings
    .write_to_file(out_path.join("inline.rs"))
    .expect("Couldn't write bindings!");
}
