# recastnavigation-sys

Raw Rust bindings for `recastnavigation`, including Recast, Detour, DetourCrowd, and DetourTileCache.

## Structure of the crate

These are the raw FFI bindings for all libraries in `recastnavigation`. Each library is enabled by a feature flag with the corresponding name:

* `recast` -> Recast
* `detour` -> Detour
* `detour_crowd` -> DetourCrowd
* `detour_tile_cache` -> DetourTileCache

In addition to the library feature flags are the "extra features". Currently, these are:

* `detour_large_nav_meshes` - enables 64-bit dtPolyRefs, allowing for larger navigation meshes in Detour (and its dependents). Note this means wrappers around Detour should be aware that dtPolyRefs can have different sizes and either have a static assertion of the expected dtPolyRef size, or ensure that your wrapper can handle both dtPolyRef sizes.

## Finding vs vendoring

By default, this crate will first look for `recastnavigation` using pkgconfig. If `recastnavigation` cannot be found, this crate will build it from the `recastnavigation` directory (aka vendoring).

To always vendor, set `RECAST_VENDOR=true` in your environment variables. This will not search for a system-installed `recastnavigation` and instead will always build `recastnavigation` from the source in this crate.

To never vendor, set `RECAST_NO_VENDOR=true` in your environment variables. This will search for a system-installed `recastnavigation` and will fail to build this crate if it is missing. Note this takes precedence over `RECAST_VENDOR=true`.

## Versioning

Unfortunately, `recastnavigation` does not have a regular release process. First this means that requiring a certain version from pkgconfig is unreliable - hopefully, compilation errors will make it clear that `recastnavigation` is stale. Second, this crate's version number cannot really track `recastnavigation`'s version. The goal is to track `recastnavigation` changes using semantic versioning, although practically not every commit to `recastnavigation` will correspond to a version of this crate. Bumping `recastnavigation` should be requested with an issue if necessary.

## License

Licensed under the [MIT license](LICENSE). This does not extend to the recastnavigation directory, which contains the contents of the [recastnavigation](https://github.com/recastnavigation/recastnavigation) project (as a Git submodule).
