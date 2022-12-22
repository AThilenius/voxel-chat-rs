use bevy::{prelude::*, render::render_resource::PrimitiveTopology};
use hibitset::BitSet;

use super::{Buffer, PbrProps, LN_SIZE, WIDTH};
use std::mem::swap;

/// # Greedy meshing for an entire Buffer into a single mesh.
///
/// ## How it works
/// Meshing is done in chunks, but all output is combined. Tessellating a single chunk is done in 2D
/// slices, one for each of the 6 cardinal directions. However, all 6 directions share the same basic
/// meshing logic, the only thing that changes is how the 2D plane is 'mapped' into 3D space. This
/// transform is refereed to as as the 'Vector Space Transform' (or VST). Each slice is meshed
/// 'greedily' meaning it will try to combine adjacent like-material faces into a single quad.
///
/// Additionally, while meshing, the adjacent voxel (towards the face normal) is checked to see if
/// it's opaque. Opaque voxels that occlude and entire quad will result in the quad being culled.
///
/// TODO: This next part isn't true... yet...
/// However, opaque voxels that occlude only the middle part of a quad will NOT cull or split the
/// quad. This reduces the number of verts, but can cause overdraw. To combat overdraw, quads should
/// be meshed in a 'closest to furthest' order for each cardinal direction (ex, the +X direction
/// should be meshed from smallest to largest). Along a chunk boundary, this lookup lambda will
/// either query voxels from the adjacent chunk, or return a const `Translucent` value.
///
/// This approach has the added benefit of massively reducing the number of chunk lookups, resulting
/// in fantastic performance given that the slowest part of voxel lookup in a Buffer is the hashmap
/// lookup of the chunk.
///
/// The resultant mesh is indexed, with 4 verts per quad. Common like-material verts are NOT
/// combined however. This every so slightly increases the number of verts, but means that all
/// tessellated meshes share the exact same static index buffer: [0, 1, 2, 0, 2, 3, ...]
impl From<Buffer> for Mesh {
    fn from(buffer: Buffer) -> Self {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        //
        // TODO
        //
        mesh
    }
}

fn mesh_2d_slice<F>(
    // Lambda that converts from 2D 'slice space' into 3D 'voxel space' in-plane with the slice
    // currently being meshed.
    slice_vst: F,

    // Much like `slice_vst` but one slice 'back' (in the direction of the normal of the 2D slice).
    normal_vst: F,
) where
    F: Fn(UVec2) -> PbrProps,
{
    // Negative mask; a set bit in the mask represents a voxel that cannot be meshed, or is already
    // a member of another quad.
    let mut mask = BitSet::with_capacity((WIDTH * WIDTH) as u32);

    for v in 0..(WIDTH as u32) {
        for u in 0..(WIDTH as u32) {
            let i = (v << LN_SIZE) + u;

            // Check if it's an already meshed voxel
            if mask.contains(i) {
                continue;
            }

            // The PbrProps type for this quad.
            let p = slice_vst((u, v).into());

            // Greedily consume first right, then upward, extending out the size of the quad as much
            // as we can.
            let mut quad_size = UVec2::ONE;

            // Extend right as far as we can.
            for u in u..(WIDTH as u32) {
                let i = (v << LN_SIZE) + u;

                if mask.contains(i)
                    || slice_vst((u, v).into()) != p
                    || normal_vst((u, v).into()).is_opaque()
                {
                    break;
                }

                mask.add(i);
                quad_size.x += 1;
            }

            // Extend upward as far as we can (entire width has to fit each time)
            'outer: for v in v..(WIDTH as u32) {
                // First check that the entire width can be added. If it can't be, then we are done
                // extending upward entirely.
                for u in u..(u + quad_size.x) {
                    let i = (v << LN_SIZE) + u;
                    if mask.contains(i)
                        || slice_vst((u, v).into()) != p
                        || normal_vst((u, v).into()).is_opaque()
                    {
                        break 'outer;
                    }
                }

                // It can be, add it in
                for u in u..(u + quad_size.x) {
                    let i = (v << LN_SIZE) + u;
                    mask.add(i);
                }

                quad_size.y += 1;
            }

            // Done with the greedy part. We now have a quad that extends from uv to (uv +
            // quad_size).
        }
    }

    let test = slice_opaque(UVec2::ZERO);
}
