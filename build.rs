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
  fn create_bindings(
    add_to_builder: impl Fn(bindgen::Builder) -> bindgen::Builder,
    out_file: PathBuf,
  ) {
    let builder = bindgen::Builder::default()
      .parse_callbacks(Box::new(bindgen::CargoCallbacks))
      .clang_args(["-x", "c++"].iter())
      .blocklist_file(".*stddef\\.h")
      .blocklist_type("max_align_t");

    let bindings =
      add_to_builder(builder).generate().expect("Unable to generate bindings.");

    bindings.write_to_file(out_file).expect("Couldn't write bindings!");
  }

  let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

  create_bindings(
    |builder| builder.header("recastnavigation/Recast/Include/Recast.h"),
    out_path.join("recast.rs"),
  );

  create_bindings(
    |builder| {
      builder
        .header("recastnavigation/Detour/Include/DetourAlloc.h")
        .header("recastnavigation/Detour/Include/DetourStatus.h")
        .header("recastnavigation/Detour/Include/DetourNavMesh.h")
        .header("recastnavigation/Detour/Include/DetourNavMeshBuilder.h")
        .header("recastnavigation/Detour/Include/DetourNavMeshQuery.h")
    },
    out_path.join("detour.rs"),
  );

  create_bindings(
    |builder| {
      builder
        .header("recastnavigation/DetourCrowd/Include/DetourCrowd.h")
        .blocklist_file(".*DetourAlloc\\.h")
        .blocklist_file(".*DetourNavMesh\\.h")
        .blocklist_file(".*DetourNavMeshQuery\\.h")
        .blocklist_file(".*DetourStatus\\.h")
        .clang_args(["-Irecastnavigation/Detour/Include"].iter())
    },
    out_path.join("detour_crowd.rs"),
  );

  create_bindings(
    |builder| {
      builder
        .header("recastnavigation/DetourTileCache/Include/DetourTileCache.h")
        .header(
          "recastnavigation/DetourTileCache/Include/DetourTileCacheBuilder.h",
        )
        .blocklist_file(".*DetourAlloc\\.h")
        .blocklist_file(".*DetourStatus\\.h")
        .blocklist_type("dtNavMesh")
        .blocklist_type("dtNavMeshCreateParams")
        .clang_args(["-Irecastnavigation/Detour/Include"].iter())
    },
    out_path.join("detour_tile_cache.rs"),
  )
}

fn build_and_link_inline_lib() {
  println!("cargo:rerun-if-changed=inline_lib_src");

  cc::Build::new()
    .file("inline_lib_src/inline.cc")
    .include("recastnavigation/Recast/Include")
    .include("recastnavigation/Detour/Include")
    .include("recastnavigation/DetourCrowd/Include")
    .include("recastnavigation/DetourTileCache/Include")
    .compile("recast_inline");

  println!("cargo:rustc-link-search=native={}", env::var("OUT_DIR").unwrap());
  println!("cargo:rustc-link-lib=static=recast_inline");
}

fn generate_inline_bindings() {
  let bindings = bindgen::Builder::default()
    .header("inline_lib_src/inline.h")
    .parse_callbacks(Box::new(bindgen::CargoCallbacks))
    .clang_args(
      [
        "-x",
        "c++",
        "-Irecastnavigation/Recast/Include",
        "-Irecastnavigation/Detour/Include",
        "-Irecastnavigation/DetourCrowd/Include",
        "-Irecastnavigation/DetourTileCache/Include",
      ]
      .iter(),
    )
    .allowlist_recursively(false)
    .allowlist_file("inline_lib_src/inline.h")
    .generate()
    .expect("Unable to generate bindings.");

  let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
  bindings
    .write_to_file(out_path.join("inline.rs"))
    .expect("Couldn't write bindings!");
}
