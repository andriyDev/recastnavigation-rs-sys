#[allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
mod ffi_recast {
  include!(concat!(env!("OUT_DIR"), "/recast.rs"));
}

#[allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
mod ffi_detour {
  include!(concat!(env!("OUT_DIR"), "/detour_Status.rs"));
  include!(concat!(env!("OUT_DIR"), "/detour_NavMesh.rs"));
  include!(concat!(env!("OUT_DIR"), "/detour_NavMeshBuilder.rs"));
  include!(concat!(env!("OUT_DIR"), "/detour_NavMeshQuery.rs"));
}

mod ffi_inline {
  use crate::ffi_recast::rcContext;

  include!(concat!(env!("OUT_DIR"), "/inline.rs"));
}

pub use ffi_detour::*;
pub use ffi_inline::*;
pub use ffi_recast::*;

#[cfg(test)]
mod tests {
  use crate::{ffi_inline::*, ffi_recast::*};

  #[test]
  fn create_simple_nav_mesh() {
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
}
