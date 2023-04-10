#[allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
mod ffi_recast {
  include!(concat!(env!("OUT_DIR"), "/recast.rs"));
}

#[allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
mod ffi_detour {
  include!(concat!(env!("OUT_DIR"), "/detour.rs"));
}

#[allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
mod ffi_detour_crowd {
  use crate::ffi_detour::*;

  include!(concat!(env!("OUT_DIR"), "/detour_crowd.rs"));
}

#[allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
mod ffi_detour_tile_cache {
  use crate::ffi_detour::*;

  include!(concat!(env!("OUT_DIR"), "/detour_tile_cache.rs"));
}

#[allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
mod ffi_inline {
  use crate::ffi_detour::*;
  use crate::ffi_detour_tile_cache::*;
  use crate::ffi_recast::*;

  include!(concat!(env!("OUT_DIR"), "/inline.rs"));
}

pub use ffi_detour::*;
pub use ffi_detour_crowd::*;
pub use ffi_detour_tile_cache::*;
pub use ffi_inline::*;
pub use ffi_recast::*;

#[cfg(test)]
mod tests {
  use crate::{
    ffi_detour::*, ffi_detour_crowd::*, ffi_detour_tile_cache::*,
    ffi_inline::*, ffi_recast::*,
  };

  #[test]
  fn recast_create_simple_nav_mesh() {
    let context = unsafe { CreateContext(false) };
    let heightfield = unsafe { rcAllocHeightfield() };

    unsafe {
      rcCreateHeightfield(
        context,
        heightfield,
        5,
        5,
        [0.0, 0.0, 0.0].as_ptr(),
        [5.0, 5.0, 5.0].as_ptr(),
        1.0,
        1.0,
      )
    };

    let verts = [
      0.0, 0.5, 0.0, //
      5.0, 0.5, 0.0, //
      5.0, 0.5, 5.0, //
      0.0, 0.5, 5.0, //
    ];
    let triangles: &[i32] = &[0, 1, 2, 2, 3, 0];
    let area_ids: &[u8] = &[RC_WALKABLE_AREA, RC_WALKABLE_AREA];

    assert!(
      unsafe {
        rcRasterizeTriangles(
          context,
          verts.as_ptr(),
          verts.len() as i32 / 3,
          triangles.as_ptr(),
          area_ids.as_ptr(),
          triangles.len() as i32 / 3,
          heightfield,
          1,
        )
      },
      "Expected rasterization to succeed."
    );

    let compact_heightfield = unsafe { rcAllocCompactHeightfield() };
    assert!(unsafe {
      rcBuildCompactHeightfield(context, 2, 1, heightfield, compact_heightfield)
    });
    unsafe { rcFreeHeightField(heightfield) };

    assert!(unsafe { rcErodeWalkableArea(context, 1, compact_heightfield) });
    assert!(unsafe { rcBuildDistanceField(context, compact_heightfield) });
    assert!(unsafe {
      rcBuildRegions(
        context,
        compact_heightfield,
        /*borderSize=*/ 0,
        /*minRegionArea=*/ 0,
        /*mergeRegionArea=*/ 0,
      )
    });

    let contour_set = unsafe { rcAllocContourSet() };

    assert!(unsafe {
      rcBuildContours(
        context,
        compact_heightfield,
        /*maxError=*/ 0.0,
        /*maxEdgeLen=*/ 0,
        contour_set,
        rcBuildContoursFlags_RC_CONTOUR_TESS_WALL_EDGES,
      )
    });

    unsafe { rcFreeCompactHeightfield(compact_heightfield) };

    let mesh = unsafe { rcAllocPolyMesh() };
    assert!(unsafe {
      rcBuildPolyMesh(context, contour_set, /*nvp=*/ 8, mesh)
    });
    unsafe { rcFreeContourSet(contour_set) };
    unsafe { DeleteContext(context) };

    let mesh_ref = unsafe { &mut *mesh };

    assert_eq!(mesh_ref.npolys, 1);
    assert_eq!(mesh_ref.nverts, 4);
    let node_slice = unsafe {
      std::slice::from_raw_parts(
        mesh_ref.polys,
        (mesh_ref.npolys * mesh_ref.nvp * 2) as usize,
      )
    };
    assert_eq!(
      &node_slice[..(mesh_ref.npolys * mesh_ref.nvp) as usize],
      &[
        0,
        1,
        2,
        3,
        RC_MESH_NULL_IDX,
        RC_MESH_NULL_IDX,
        RC_MESH_NULL_IDX,
        RC_MESH_NULL_IDX,
      ]
    );
    let vert_slice = unsafe {
      std::slice::from_raw_parts(mesh_ref.verts, mesh_ref.nverts as usize * 3)
    };
    let expected_verts = &[
      1, 1, 1, //
      1, 1, 4, //
      4, 1, 4, //
      4, 1, 1, //
    ];
    assert_eq!(vert_slice, expected_verts);

    unsafe { rcFreePolyMesh(mesh) };
  }

  #[test]
  fn detour_finds_simple_path() {
    let verts = vec![
      1, 0, 1, //
      1, 0, 0, //
      2, 0, 0, //
      2, 0, 1, //
      3, 0, 1, //
      3, 0, 2, //
      2, 0, 2, //
      1, 0, 2, //
      0, 0, 2, //
      0, 0, 1, //
    ];

    const N: u16 = RC_MESH_NULL_IDX;

    let polys = vec![
      0, 1, 2, N, N, 1, //
      2, 3, 0, N, 2, 0, //
      0, 3, 6, 1, 4, 3, //
      0, 6, 7, 2, N, 6, //
      6, 3, 4, 2, N, 5, //
      6, 4, 5, 4, N, N, //
      0, 7, 8, 3, N, 7, //
      0, 8, 9, 6, N, N, //
    ];

    let poly_flags = vec![1, 1, 1, 1, 1, 1, 1, 1];
    let poly_areas = vec![0, 0, 0, 0, 0, 0, 0, 0];

    let mut nav_mesh_create_data = dtNavMeshCreateParams {
      verts: verts.as_ptr(),
      vertCount: verts.len() as i32,
      polys: polys.as_ptr(),
      polyFlags: poly_flags.as_ptr(),
      polyAreas: poly_areas.as_ptr(),
      polyCount: polys.len() as i32 / 6,
      nvp: 3,
      detailMeshes: std::ptr::null(),
      detailVerts: std::ptr::null(),
      detailVertsCount: 0,
      detailTris: std::ptr::null(),
      detailTriCount: 0,
      offMeshConVerts: std::ptr::null(),
      offMeshConRad: std::ptr::null(),
      offMeshConFlags: std::ptr::null(),
      offMeshConAreas: std::ptr::null(),
      offMeshConDir: std::ptr::null(),
      offMeshConUserID: std::ptr::null(),
      offMeshConCount: 0,
      userId: 0,
      tileX: 0,
      tileY: 0,
      tileLayer: 0,
      bmin: [0.0, 0.0, 0.0],
      bmax: [3.0, 2.0, 2.0],
      walkableHeight: 1.0,
      walkableRadius: 1.0,
      walkableClimb: 1.0,
      cs: 1.0,
      ch: 1.0,
      buildBvTree: false,
    };

    let mut data: *mut u8 = std::ptr::null_mut();
    let mut data_size: i32 = 0;

    assert!(unsafe {
      dtCreateNavMeshData(&mut nav_mesh_create_data, &mut data, &mut data_size)
    });

    let nav_mesh = unsafe { &mut *dtAllocNavMesh() };
    assert_ne!(nav_mesh as *mut dtNavMesh, std::ptr::null_mut());
    assert_eq!(
      unsafe { nav_mesh.init1(data, data_size, dtTileFlags_DT_TILE_FREE_DATA) },
      DT_SUCCESS
    );

    let nav_mesh_query = unsafe { &mut *dtAllocNavMeshQuery() };
    assert_ne!(nav_mesh as *mut dtNavMesh, std::ptr::null_mut());
    assert_eq!(unsafe { nav_mesh_query.init(nav_mesh, 512) }, DT_SUCCESS);

    fn find_poly_ref(query: &dtNavMeshQuery, pos: &[f32; 3]) -> dtPolyRef {
      let extents = [0.1, 100.0, 0.1];

      let mut poly_ref: dtPolyRef = 0;

      let query_filter = dtQueryFilter {
        m_areaCost: [1.0; 64],
        m_includeFlags: 0xffff,
        m_excludeFlags: 0,
      };

      assert_eq!(
        unsafe {
          query.findNearestPoly(
            pos.as_ptr(),
            extents.as_ptr(),
            &query_filter,
            &mut poly_ref,
            std::ptr::null_mut(),
          )
        },
        DT_SUCCESS
      );

      poly_ref
    }

    let start_point = [1.1, 0.0, 0.1];
    let end_point = [2.9, 0.0, 1.9];

    let start_poly_ref = find_poly_ref(&nav_mesh_query, &start_point);
    let end_poly_ref = find_poly_ref(&nav_mesh_query, &end_point);

    assert_eq!(start_poly_ref, 8);
    assert_eq!(end_poly_ref, 13);

    let query_filter = dtQueryFilter {
      m_areaCost: [1.0; 64],
      m_includeFlags: 0xffff,
      m_excludeFlags: 0,
    };

    let mut path = [0; 10];
    let mut path_count = 0;

    assert_eq!(
      unsafe {
        nav_mesh_query.findPath(
          start_poly_ref,
          end_poly_ref,
          start_point.as_ptr(),
          end_point.as_ptr(),
          &query_filter,
          path.as_mut_ptr(),
          &mut path_count,
          path.len() as i32,
        )
      },
      DT_SUCCESS
    );

    assert_eq!(path, [8, 9, 10, 12, 13, 0, 0, 0, 0, 0]);

    unsafe { dtFreeNavMeshQuery(nav_mesh_query) };
    unsafe { dtFreeNavMesh(nav_mesh) };
  }

  #[test]
  fn detour_crowd_basic_path_following() {
    let verts = vec![
      1, 0, 1, //
      1, 0, 0, //
      2, 0, 0, //
      2, 0, 1, //
      3, 0, 1, //
      3, 0, 2, //
      2, 0, 2, //
      1, 0, 2, //
      0, 0, 2, //
      0, 0, 1, //
    ];

    const N: u16 = RC_MESH_NULL_IDX;

    let polys = vec![
      0, 1, 2, N, N, 1, //
      2, 3, 0, N, 2, 0, //
      0, 3, 6, 1, 4, 3, //
      0, 6, 7, 2, N, 6, //
      6, 3, 4, 2, N, 5, //
      6, 4, 5, 4, N, N, //
      0, 7, 8, 3, N, 7, //
      0, 8, 9, 6, N, N, //
    ];

    let poly_flags = vec![1, 1, 1, 1, 1, 1, 1, 1];
    let poly_areas = vec![0, 0, 0, 0, 0, 0, 0, 0];

    let mut nav_mesh_create_data = dtNavMeshCreateParams {
      verts: verts.as_ptr(),
      vertCount: verts.len() as i32,
      polys: polys.as_ptr(),
      polyFlags: poly_flags.as_ptr(),
      polyAreas: poly_areas.as_ptr(),
      polyCount: polys.len() as i32 / 6,
      nvp: 3,
      detailMeshes: std::ptr::null(),
      detailVerts: std::ptr::null(),
      detailVertsCount: 0,
      detailTris: std::ptr::null(),
      detailTriCount: 0,
      offMeshConVerts: std::ptr::null(),
      offMeshConRad: std::ptr::null(),
      offMeshConFlags: std::ptr::null(),
      offMeshConAreas: std::ptr::null(),
      offMeshConDir: std::ptr::null(),
      offMeshConUserID: std::ptr::null(),
      offMeshConCount: 0,
      userId: 0,
      tileX: 0,
      tileY: 0,
      tileLayer: 0,
      bmin: [0.0, 0.0, 0.0],
      bmax: [3.0, 2.0, 2.0],
      walkableHeight: 1.0,
      walkableRadius: 1.0,
      walkableClimb: 1.0,
      cs: 1.0,
      ch: 1.0,
      buildBvTree: false,
    };

    let mut data: *mut u8 = std::ptr::null_mut();
    let mut data_size: i32 = 0;

    assert!(unsafe {
      dtCreateNavMeshData(&mut nav_mesh_create_data, &mut data, &mut data_size)
    });

    let nav_mesh = unsafe { &mut *dtAllocNavMesh() };
    assert_ne!(nav_mesh as *mut dtNavMesh, std::ptr::null_mut());
    assert_eq!(
      unsafe { nav_mesh.init1(data, data_size, dtTileFlags_DT_TILE_FREE_DATA) },
      DT_SUCCESS
    );

    let crowd = unsafe { &mut *dtAllocCrowd() };
    assert!(unsafe { crowd.init(10, 10.0, nav_mesh) });

    let agent_position = [1.1, 0.0, 0.1];
    let agent_params = dtCrowdAgentParams {
      radius: 0.25,
      height: 1.0,
      maxAcceleration: 0.5,
      maxSpeed: 1.0,
      collisionQueryRange: 1.0,
      pathOptimizationRange: 10.0,
      separationWeight: 1.0,
      updateFlags: 0,
      obstacleAvoidanceType: 0,
      queryFilterType: 0,
      userData: std::ptr::null_mut(),
    };

    assert_eq!(
      unsafe { crowd.addAgent(agent_position.as_ptr(), &agent_params) },
      0
    );

    unsafe { crowd.update(0.1, std::ptr::null_mut()) };

    let updated_position = unsafe { &(*crowd.getAgent(0)).npos };
    assert_eq!(updated_position, &agent_position);

    let target_point = [2.9, 0.0, 1.9];
    let extents = [0.1, 100.0, 0.1];

    let mut target_poly_ref: dtPolyRef = 0;

    let query_filter = dtQueryFilter {
      m_areaCost: [1.0; 64],
      m_includeFlags: 0xffff,
      m_excludeFlags: 0,
    };

    assert_eq!(
      unsafe {
        (*crowd.m_navquery).findNearestPoly(
          target_point.as_ptr(),
          extents.as_ptr(),
          &query_filter,
          &mut target_poly_ref,
          std::ptr::null_mut(),
        )
      },
      DT_SUCCESS
    );

    assert!(unsafe {
      crowd.requestMoveTarget(0, target_poly_ref, target_point.as_ptr())
    });

    for _ in 0..200 {
      unsafe { crowd.update(0.1, std::ptr::null_mut()) };
    }

    let updated_position = unsafe { &(*crowd.getAgent(0)).npos };
    let delta = [
      updated_position[0] - target_point[0],
      updated_position[1] - target_point[1],
      updated_position[2] - target_point[2],
    ];
    assert!(
      (delta[0] * delta[0] + delta[1] * delta[1] + delta[2] * delta[2]).sqrt()
        < 0.01,
      "\n\nleft: {:?}\nright: {:?}",
      updated_position,
      target_point
    );

    unsafe { dtFreeCrowd(crowd) };
    unsafe { dtFreeNavMesh(nav_mesh) };
  }

  #[test]
  fn detour_tile_cache_simple_caching() {
    let cache_params = dtTileCacheParams {
      orig: [0.0, 0.0, 0.0],
      cs: 1.0,
      ch: 1.0,
      width: 5,
      height: 5,
      walkableHeight: 1.0,
      walkableRadius: 1.0,
      walkableClimb: 1.0,
      maxSimplificationError: 0.01,
      maxTiles: 1000,
      maxObstacles: 10,
    };

    let alloc = unsafe { CreateDefaultTileCacheAlloc() };

    extern "C" fn max_compressed_size(
      _object_ptr: *mut std::ffi::c_void,
      buffer_size: i32,
    ) -> i32 {
      buffer_size
    }

    extern "C" fn compress(
      _object_ptr: *mut std::ffi::c_void,
      buffer: *const u8,
      buffer_size: i32,
      compressed: *mut u8,
      max_compressed_size: i32,
      compressed_size: *mut i32,
    ) -> u32 {
      assert!(
        buffer_size <= max_compressed_size,
        "\n\nleft: {}\nright: {}",
        buffer_size,
        max_compressed_size
      );

      let buffer_slice =
        unsafe { std::slice::from_raw_parts(buffer, buffer_size as usize) };

      unsafe { *compressed_size = buffer_size };

      let compressed_slice = unsafe {
        std::slice::from_raw_parts_mut(compressed, *compressed_size as usize)
      };

      compressed_slice.copy_from_slice(buffer_slice);

      DT_SUCCESS
    }

    extern "C" fn decompress(
      object_ptr: *mut std::ffi::c_void,
      compressed: *const u8,
      compressed_size: i32,
      buffer: *mut u8,
      max_buffer_size: i32,
      buffer_size: *mut i32,
    ) -> u32 {
      // Since compress just copies the source to destination, decompress is
      // the exact same.
      compress(
        object_ptr,
        compressed,
        compressed_size,
        buffer,
        max_buffer_size,
        buffer_size,
      )
    }
    let forwarded_compressor = unsafe {
      CreateForwardedTileCacheCompressor(
        std::ptr::null_mut(),
        Some(max_compressed_size),
        Some(compress),
        Some(decompress),
      )
    };

    extern "C" fn set_poly_flags(
      _: *mut std::ffi::c_void,
      params: *mut dtNavMeshCreateParams,
      _areas: *mut u8,
      flags: *mut u16,
    ) {
      let params = unsafe { &*params };
      let flags = unsafe {
        std::slice::from_raw_parts_mut(flags, params.polyCount as usize)
      };

      flags.fill(1);
    }
    let forwarded_mesh_process = unsafe {
      CreateForwardedTileCacheMeshProcess(
        std::ptr::null_mut(),
        Some(set_poly_flags),
      )
    };

    let tile_cache = unsafe { &mut *dtAllocTileCache() };
    unsafe {
      tile_cache.init(
        &cache_params,
        alloc,
        forwarded_compressor,
        forwarded_mesh_process,
      )
    };

    let nav_mesh = unsafe { &mut *dtAllocNavMesh() };
    let nav_mesh_params = dtNavMeshParams {
      orig: [0.0, 0.0, 0.0],
      tileWidth: 5.0,
      tileHeight: 5.0,
      maxTiles: 1000,
      maxPolys: 10,
    };
    unsafe { nav_mesh.init(&nav_mesh_params) };

    for _ in 0..10 {
      let mut up_to_date = false;
      unsafe { tile_cache.update(1.0, nav_mesh, &mut up_to_date) };
      assert!(up_to_date);
    }

    let mut header = dtTileCacheLayerHeader {
      magic: DT_TILECACHE_MAGIC,
      version: DT_TILECACHE_VERSION,
      tx: 0,
      ty: 0,
      tlayer: 0,
      bmin: [0.0, 1.0, 0.0],
      bmax: [5.0, 1.0, 5.0],
      width: 5 as u8,
      height: 5 as u8,
      minx: 0,
      maxx: 4,
      miny: 0,
      maxy: 4,
      hmin: 1,
      hmax: 1,
    };

    const N: u8 = 255;

    let heights = [
      N, N, 0, N, N, //
      N, N, 0, N, N, //
      N, N, 0, N, N, //
      N, N, 0, N, N, //
      0, 0, 0, 0, 0, //
    ];

    const W: u8 = DT_TILECACHE_WALKABLE_AREA;

    let areas = [
      0, 0, W, 0, 0, //
      0, 0, W, 0, 0, //
      0, 0, W, 0, 0, //
      0, 0, W, 0, 0, //
      W, W, W, W, W, //
    ];

    // Neighbour connectivity.
    let cons = [
      0, 0, 2, 0, 0, //
      0, 0, 10, 0, 0, //
      0, 0, 10, 0, 0, //
      0, 0, 10, 0, 0, //
      4, 5, 13, 5, 1, //
    ];

    let mut data: *mut u8 = std::ptr::null_mut();
    let mut data_size: i32 = 0;

    assert_eq!(
      unsafe {
        dtBuildTileCacheLayer(
          forwarded_compressor,
          &mut header,
          heights.as_ptr(),
          areas.as_ptr(),
          cons.as_ptr(),
          &mut data,
          &mut data_size,
        )
      },
      DT_SUCCESS
    );

    assert_eq!(
      unsafe {
        tile_cache.addTile(
          data,
          data_size,
          dtTileFlags_DT_TILE_FREE_DATA as u8,
          std::ptr::null_mut(),
        )
      },
      DT_SUCCESS
    );

    assert_eq!(
      unsafe { tile_cache.buildNavMeshTilesAt(0, 0, nav_mesh) },
      DT_SUCCESS
    );

    let query = unsafe { &mut *dtAllocNavMeshQuery() };
    assert_eq!(unsafe { query.init(nav_mesh, 10) }, DT_SUCCESS);

    let query_filter = dtQueryFilter {
      m_areaCost: [1.0; 64],
      m_includeFlags: 0xffff,
      m_excludeFlags: 0,
    };

    let mut path = [0; 10];
    let mut path_count = 0;

    let start_point = [2.1, 1.0, 0.1];
    let end_point = [4.9, 1.0, 4.9];

    let mut start_point_ref = 0;
    assert_eq!(
      unsafe {
        query.findNearestPoly(
          start_point.as_ptr(),
          [0.1, 100.0, 0.1].as_ptr(),
          &query_filter,
          &mut start_point_ref,
          std::ptr::null_mut(),
        )
      },
      DT_SUCCESS
    );
    assert_ne!(start_point_ref, 0);

    let mut end_point_ref = 0;
    assert_eq!(
      unsafe {
        query.findNearestPoly(
          end_point.as_ptr(),
          [0.1, 100.0, 0.1].as_ptr(),
          &query_filter,
          &mut end_point_ref,
          std::ptr::null_mut(),
        )
      },
      DT_SUCCESS
    );
    assert_ne!(end_point_ref, 0);

    assert_eq!(
      unsafe {
        query.findPath(
          start_point_ref,
          end_point_ref,
          start_point.as_ptr(),
          end_point.as_ptr(),
          &query_filter,
          path.as_mut_ptr(),
          &mut path_count,
          path.len() as i32,
        )
      },
      DT_SUCCESS
    );

    assert_eq!(path, [16385, 16387, 16384, 0, 0, 0, 0, 0, 0, 0]);

    unsafe { dtFreeNavMeshQuery(query) };
    unsafe { dtFreeNavMesh(nav_mesh) };
    unsafe { dtFreeTileCache(tile_cache) };
    unsafe { DeleteTileCacheMeshProcess(forwarded_mesh_process) };
    unsafe { DeleteTileCacheCompressor(forwarded_compressor) };
    unsafe { DeleteTileCacheAlloc(alloc) };
  }
}
