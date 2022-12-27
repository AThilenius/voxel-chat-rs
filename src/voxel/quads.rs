use bevy::prelude::*;

use super::{Buffer, PbrProps, WorldCoord, LN_SIZE, WIDTH};

// Last valid index in a chunk's width.
const W: i32 = WIDTH as i32 - 1;

/// All 6 vector spaces, in a tuple of (origin, normal, tangent, bi-tangent).
const VECTOR_SPACES: [(
    bevy::prelude::IVec3,
    bevy::prelude::IVec3,
    bevy::prelude::IVec3,
    bevy::prelude::IVec3,
); 6] = [
    (IVec3::new(W, 0, W), IVec3::X, IVec3::NEG_Z, IVec3::Y),
    (IVec3::new(0, 0, 0), IVec3::NEG_X, IVec3::Z, IVec3::Y),
    (IVec3::new(0, W, 0), IVec3::Y, IVec3::Z, IVec3::X),
    (IVec3::new(0, 0, W), IVec3::NEG_Y, IVec3::NEG_Z, IVec3::X),
    (IVec3::new(0, 0, W), IVec3::Z, IVec3::NEG_X, IVec3::Y),
    (IVec3::new(W, 0, 0), IVec3::NEG_Z, IVec3::NEG_X, IVec3::Y),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Vert {
    pub pos: WorldCoord,
    pub uv: UVec2,
}

pub struct Quad {
    pub verts: [Vert; 4],
    pub pbr_props: PbrProps,
    pub normal: IVec3,
    pub tangent: IVec3,
}

impl Quad {
    /// Tesselates a buffer into a vec of quads.
    pub fn from_buffer(buffer: &Buffer) -> Vec<Quad> {
        let mut quads = Vec::new();

        for chunk_coord in buffer.chunks.keys() {
            let chunk_origin = chunk_coord.first_cell_coord().0;

            for (origin_offset, normal, tan, bi_tan) in VECTOR_SPACES {
                let origin = chunk_origin + origin_offset;

                for slice in 0..(WIDTH as i32) {
                    let slice_origin = origin + (-normal * slice);

                    // Negative mask; a set bit in the mask represents a voxel that cannot be
                    // meshed, or is already a member of another quad.
                    let mut mask = [false; WIDTH * WIDTH];

                    for v in 0..(WIDTH as u32) {
                        for u in 0..(WIDTH as u32) {
                            let i = (v << LN_SIZE) + u;
                            let coord = slice_origin + (tan * u as i32) + (bi_tan * v as i32);

                            // The PbrProps type for this quad.
                            let p = buffer.get(WorldCoord(coord));

                            // Check if it's an already meshed voxel or is itself transparent
                            if mask[i as usize] || p.color.a == 0 {
                                continue;
                            }

                            // Greedily consume first right, then upward, extending out the size of
                            // the quad as much as we can.
                            let mut quad_size = UVec2::ONE;

                            // Extend right as far as we can.
                            for u in u..(WIDTH as u32) {
                                let i = (v << LN_SIZE) + u;
                                let world_coord = WorldCoord(coord + (tan * u as i32));
                                let normal_coord = WorldCoord(world_coord.0 + normal);

                                if mask[i as usize]
                                    || buffer.get(world_coord) != p
                                    || buffer.get(normal_coord).color.a == 255
                                {
                                    break;
                                }

                                mask[i as usize] = true;
                                quad_size.x += 1;
                            }

                            // Extend upward as far as we can (entire width has to fit each time)
                            'outer: for v in v..(WIDTH as u32) {
                                // First check that the entire width can be added. If it can't be,
                                // then we are done extending upward entirely.
                                for u in u..(u + quad_size.x) {
                                    let i = (v << LN_SIZE) + u;
                                    let world_coord =
                                        WorldCoord(coord + (tan * u as i32) + (bi_tan * v as i32));
                                    let normal_coord = WorldCoord(world_coord.0 + normal);

                                    if mask[i as usize]
                                        || buffer.get(world_coord) != p
                                        || buffer.get(normal_coord).color.a == 255
                                    {
                                        break 'outer;
                                    }
                                }

                                // It can be, add it in
                                for u in u..(u + quad_size.x) {
                                    let i = (v << LN_SIZE) + u;
                                    mask[i as usize] = true
                                }

                                quad_size.y += 1;
                            }

                            // Done with the greedy part. We now have a quad that extends from uv to
                            // (uv + quad_size).
                            let pos = coord + (tan * u as i32) + (bi_tan * v as i32);
                            let right = tan * quad_size.x as i32;
                            let up = bi_tan * quad_size.y as i32;

                            info!("Meshing quad at {:?} with size {:?}", pos, quad_size,);

                            [
                                (pos, UVec2::ZERO),
                                (pos + right, UVec2::new(quad_size.x, 0)),
                                (pos + up, UVec2::new(0, quad_size.y)),
                                (pos + right, UVec2::new(quad_size.x, 0)),
                                (pos + right + up, UVec2::new(quad_size.x, quad_size.y)),
                                (pos + up, UVec2::new(0, quad_size.y)),
                            ]
                            .into_iter()
                            .for_each(|(pos, uv)| {
                                quads.push(Vert {
                                    pbr_props: p,
                                    pos: WorldCoord(pos),
                                    uv,
                                    normal,
                                    tangent: tan,
                                });
                            });
                        }
                    }
                }
            }
        }

        quads
    }
}
