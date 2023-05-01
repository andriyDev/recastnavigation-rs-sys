use std::{collections::HashMap, env, path::PathBuf};

use cmake::Config;

fn main() {
  println!("cargo:rerun-if-env-changed=PROFILE");
  println!("cargo:rerun-if-env-changed=RECAST_NO_VENDOR");
  println!("cargo:rerun-if-env-changed=RECAST_VENDOR");

  let lib_destination;
  let include_dirs;
  let defines;
  if env::var("RECAST_NO_VENDOR").unwrap_or("false".into()) == "true" {
    (lib_destination, include_dirs, defines) = find_recast().unwrap();
  } else if env::var("RECAST_VENDOR").unwrap_or("false".into()) == "true" {
    (lib_destination, include_dirs, defines) = build_recast();
  } else {
    (lib_destination, include_dirs, defines) =
      find_recast().unwrap_or_else(|| build_recast());
  }

  println!("cargo:rustc-link-search=native={}", lib_destination.display());

  for lib in lib_names() {
    println!("cargo:rustc-link-lib=static={}", lib);
  }

  // Avoid building/linking the inlining lib if only detour/detour_crowd are
  // used (since they have no "inline" definitions).
  if cfg!(any(feature = "recast", feature = "detour_tile_cache")) {
    build_and_link_inline_lib(&include_dirs, &defines);
    generate_inline_bindings(&include_dirs, &defines);
  }

  generate_recast_bindings(&include_dirs, &defines);
}

fn is_windows() -> bool {
  env::var("CARGO_CFG_WINDOWS").is_ok()
}

fn is_debug() -> bool {
  env::var("DEBUG").unwrap() == "true"
}

fn lib_names() -> Vec<String> {
  let mut root_names = Vec::new();
  #[cfg(feature = "recast")]
  root_names.push("Recast");
  #[cfg(feature = "detour")]
  root_names.push("Detour");
  #[cfg(feature = "detour_crowd")]
  root_names.push("DetourCrowd");
  #[cfg(feature = "detour_tile_cache")]
  root_names.push("DetourTileCache");
  if is_debug() {
    root_names.iter().map(|root: &&str| root.to_string() + "-d").collect()
  } else {
    root_names.iter().map(|root: &&str| root.to_string()).collect()
  }
}

fn lib_name_to_file_name(lib_name: &str) -> String {
  if is_windows() {
    format!("{}.lib", lib_name)
  } else {
    format!("lib{}.a", lib_name)
  }
}

fn find_recast(
) -> Option<(PathBuf, Vec<PathBuf>, HashMap<String, Option<String>>)> {
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
    Some((lib_dir.clone(), lib.include_paths, lib.defines))
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

fn build_recast() -> (PathBuf, Vec<PathBuf>, HashMap<String, Option<String>>) {
  println!("cargo:rerun-if-changed=recastnavigation");
  let mut lib_builder = Config::new("recastnavigation");
  lib_builder
    .define("RECASTNAVIGATION_DEMO", "OFF")
    .define("RECASTNAVIGATION_EXAMPLES", "OFF")
    .define("RECASTNAVIGATION_TESTS", "OFF");
  #[cfg(feature = "detour_large_nav_meshes")]
  lib_builder.define("RECASTNAVIGATION_DT_POLYREF64", "ON");

  let defines = if cfg!(feature = "detour_large_nav_meshes") {
    let mut defines = HashMap::new();
    defines.insert("DT_POLYREF64".to_owned(), None);
    defines
  } else {
    HashMap::new()
  };

  let lib_destination = lib_builder.build();
  (
    lib_destination.join("lib"),
    vec![
      "recastnavigation/Recast/Include".into(),
      "recastnavigation/Detour/Include".into(),
      "recastnavigation/DetourCrowd/Include".into(),
      "recastnavigation/DetourTileCache/Include".into(),
    ],
    defines,
  )
}

fn find_in_include_dirs(
  include_dirs: &[PathBuf],
  file: &str,
) -> Option<String> {
  for include_dir in include_dirs.iter() {
    let file = include_dir.join(file);
    if file.exists() {
      return Some(file.to_string_lossy().into());
    }
  }

  None
}

fn generate_recast_bindings(
  include_dirs: &[PathBuf],
  defines: &HashMap<String, Option<String>>,
) {
  fn create_bindings(
    include_dirs: &[PathBuf],
    defines: &HashMap<String, Option<String>>,
    add_to_builder: impl Fn(bindgen::Builder) -> bindgen::Builder,
    out_file: PathBuf,
  ) {
    let builder = bindgen::Builder::default()
      .parse_callbacks(Box::new(bindgen::CargoCallbacks))
      .clang_args(["-x", "c++"].iter())
      .clang_args(
        include_dirs
          .iter()
          .map(|include_dir| format!("-I{}", include_dir.display())),
      )
      .clang_args(defines.iter().map(|(name, value)| match value {
        Some(value) => format!("-D{}={}", name, value),
        None => format!("-D{}", name),
      }))
      .blocklist_file(".*stddef\\.h")
      .blocklist_type("max_align_t");

    #[cfg(feature = "detour_large_nav_meshes")]
    let builder = builder.clang_args(["-DDT_POLYREF64"]);

    let bindings =
      add_to_builder(builder).generate().expect("Unable to generate bindings.");

    bindings.write_to_file(out_file).expect("Couldn't write bindings!");
  }

  let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

  #[cfg(feature = "recast")]
  create_bindings(
    include_dirs,
    defines,
    |builder| {
      builder.header(
        find_in_include_dirs(include_dirs, "Recast.h")
          .expect("Recast.h is present in the include dirs."),
      )
    },
    out_path.join("recast.rs"),
  );

  #[cfg(feature = "detour")]
  create_bindings(
    include_dirs,
    defines,
    |builder| {
      builder
        .header(
          find_in_include_dirs(include_dirs, "DetourAlloc.h")
            .expect("DetourAlloc.h is present in the include dirs."),
        )
        .header(
          find_in_include_dirs(include_dirs, "DetourStatus.h")
            .expect("DetourStatus.h is present in the include dirs."),
        )
        .header(
          find_in_include_dirs(include_dirs, "DetourNavMesh.h")
            .expect("DetourNavMesh.h is present in the include dirs."),
        )
        .header(
          find_in_include_dirs(include_dirs, "DetourNavMeshBuilder.h")
            .expect("DetourNavMeshBuilder.h is present in the include dirs."),
        )
        .header(
          find_in_include_dirs(include_dirs, "DetourNavMeshQuery.h")
            .expect("DetourNavMeshQuery.h is present in the include dirs."),
        )
    },
    out_path.join("detour.rs"),
  );

  #[cfg(feature = "detour_crowd")]
  create_bindings(
    include_dirs,
    defines,
    |builder| {
      builder
        .header(
          find_in_include_dirs(include_dirs, "DetourCrowd.h")
            .expect("DetourCrowd.h is present in the include dirs."),
        )
        .blocklist_file(".*DetourAlloc\\.h")
        .blocklist_file(".*DetourNavMesh\\.h")
        .blocklist_file(".*DetourNavMeshQuery\\.h")
        .blocklist_file(".*DetourStatus\\.h")
    },
    out_path.join("detour_crowd.rs"),
  );

  #[cfg(feature = "detour_tile_cache")]
  create_bindings(
    include_dirs,
    defines,
    |builder| {
      builder
        .header(
          find_in_include_dirs(include_dirs, "DetourTileCache.h")
            .expect("DetourTileCache.h is present in the include dirs."),
        )
        .header(
          find_in_include_dirs(include_dirs, "DetourTileCacheBuilder.h")
            .expect("DetourTileCacheBuilder.h is present in the include dirs."),
        )
        .blocklist_file(".*DetourAlloc\\.h")
        .blocklist_file(".*DetourStatus\\.h")
        .blocklist_type("dtNavMesh")
        .blocklist_type("dtNavMeshCreateParams")
    },
    out_path.join("detour_tile_cache.rs"),
  );
}

fn build_and_link_inline_lib(
  include_dirs: &[PathBuf],
  defines: &HashMap<String, Option<String>>,
) {
  println!("cargo:rerun-if-changed=inline_lib_src");

  let mut build = cc::Build::new();
  build.cpp(true).file("inline_lib_src/inline.cc").includes(include_dirs);

  if cfg!(feature = "recast") {
    build.define("RECAST", None);
  }
  if cfg!(feature = "detour") {
    build.define("DETOUR", None);
  }
  if cfg!(feature = "detour_crowd") {
    build.define("DETOUR_CROWD", None);
  }
  if cfg!(feature = "detour_tile_cache") {
    build.define("DETOUR_TILE_CACHE", None);
  }

  for (name, value) in defines.iter() {
    build.define(
      name,
      match value {
        Some(value) => Some(value.as_str()),
        None => None,
      },
    );
  }

  build.compile("recast_inline");

  println!("cargo:rustc-link-search=native={}", env::var("OUT_DIR").unwrap());
  println!("cargo:rustc-link-lib=static=recast_inline");
}

fn generate_inline_bindings(
  include_dirs: &[PathBuf],
  defines: &HashMap<String, Option<String>>,
) {
  let mut builder = bindgen::Builder::default()
    .header("inline_lib_src/inline.h")
    .parse_callbacks(Box::new(bindgen::CargoCallbacks))
    .clang_args(["-x", "c++"].iter())
    .clang_args(
      include_dirs
        .iter()
        .map(|include_dir| format!("-I{}", include_dir.display())),
    )
    .clang_args(defines.iter().map(|(name, value)| match value {
      Some(value) => format!("-D{}={}", name, value),
      None => format!("-D{}", name),
    }))
    .allowlist_recursively(false)
    .allowlist_file("inline_lib_src/inline.h");

  if cfg!(feature = "recast") {
    builder = builder.clang_args(["-DRECAST"].iter());
  }
  if cfg!(feature = "detour") {
    builder = builder.clang_args(["-DDETOUR"].iter());
  }
  if cfg!(feature = "detour_crowd") {
    builder = builder.clang_args(["-DDETOUR_CROWD"].iter());
  }
  if cfg!(feature = "detour_tile_cache") {
    builder = builder.clang_args(["-DDETOUR_TILE_CACHE"].iter());
  }

  #[cfg(feature = "detour_large_nav_meshes")]
  let builder = builder.clang_args(["-DDT_POLYREF64"]);

  let bindings = builder.generate().expect("Unable to generate bindings.");

  let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
  bindings
    .write_to_file(out_path.join("inline.rs"))
    .expect("Couldn't write bindings!");
}
