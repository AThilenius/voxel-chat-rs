use bevy::{prelude::*, render::render_resource::PrimitiveTopology};
use hibitset::BitSet;
use std::mem::swap;

use super::{Buffer, WIDTH};

const SLICE_VOXEL_COUNT: usize = WIDTH * WIDTH;
type Slice = [u16; SLICE_VOXEL_COUNT];

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

fn mesh_2d_slice<F>(slice_opaque: F, normal_opaque: F)
where
    F: Fn(UVec2) -> bool,
{
}
