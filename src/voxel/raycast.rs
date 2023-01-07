use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::{Buffer, Chunk, ChunkCoord, LocalCoord, WorldCoord, WIDTH};

#[derive(Debug, Clone, Copy)]
pub struct VoxelRayHit {
    /// The world coordinate of the voxel hit. (World in this context is still in the local space of
    /// the Buffer itself).
    pub world_coord: WorldCoord,

    /// The distance along the ray at which the hit occurred.
    pub distance: f32,

    /// The normal of the voxel face that was hit. This is None if the hit is within the voxel
    /// itself.
    pub normal: Option<IVec3>,
}

pub fn raycast_buffer_voxels(buffer: &Buffer, ray: Ray) -> Option<VoxelRayHit> {
    let chunk_ray_hits = raycast_chunk_coords(buffer, ray);

    for chunk_ray_hit in chunk_ray_hits {
        if let Some(voxel_hit) = raycast_chunk_voxels(
            buffer.chunks.get(&chunk_ray_hit.chunk_coord).unwrap(),
            chunk_ray_hit,
        ) {
            return Some(voxel_hit);
        }
    }

    None
}

#[derive(Debug)]
struct ChunkRayHit {
    // The chunk coordinate of the chunk hit.
    pub chunk_coord: ChunkCoord,
    // The total t of the hit along the ray, in world space. Backed off by 2 for boundary hits.
    pub t: OrderedFloat<f32>,
    // The origin of the ray in chunk-local space.
    pub origin: Vec3,
    // Same as the original ray.
    pub direction: Vec3,
}

fn raycast_chunk_coords(buffer: &Buffer, ray: Ray) -> Vec<ChunkRayHit> {
    let mut chunk_ray_hits: Vec<_> = buffer
        .chunks
        .keys()
        .map(|chunk_coord| aabb_test_chunk_coord(*chunk_coord, ray))
        .filter(|hit| hit.is_some())
        .map(|hit| hit.unwrap())
        .collect();
    chunk_ray_hits.sort_by_key(|hit| hit.t);
    chunk_ray_hits
}

/// AABB test the chunk bounds (in local space) against the ray. Returns the distance along the
/// ray to the intersection point and the intersection point itself, or None if no intersection.
/// This is relatively cheap to call.
fn aabb_test_chunk_coord(chunk_coord: ChunkCoord, ray: Ray) -> Option<ChunkRayHit> {
    let min_i = chunk_coord.first_cell_coord().0;
    let max_i = chunk_coord.last_cell_coord().0;
    let min = min_i.as_vec3();
    let max = max_i.as_vec3();
    let mut t = Vec3::ZERO;

    let Ray { origin, direction } = ray;
    let origin_i = origin.as_ivec3();

    // If the origin is withing the chunk's bounds, then we return a hit at the origin point.
    if origin_i.cmpge(min_i).all() && origin_i.cmplt(max_i).all() {
        return Some(ChunkRayHit {
            chunk_coord,
            t: OrderedFloat(0.0),
            origin,
            direction,
        });
    }

    for i in 0..3 {
        if direction[i] > 0.0 {
            t[i] = (min[i] - origin[i]) / direction[i];
        } else {
            t[i] = (max[i] - origin[i]) / direction[i];
        }
    }

    let mi = if t[0] > t[1] {
        if t[0] > t[2] {
            0
        } else {
            2
        }
    } else {
        if t[1] > t[2] {
            1
        } else {
            2
        }
    };

    if t[mi] >= 0.0 {
        // The intersect point (distance along the ray).
        let pt = origin + direction * t[mi];

        // The other two value that need to be checked
        let o1 = (mi + 1) % 3;
        let o2 = (mi + 2) % 3;

        if pt[o1] >= min[o1] && pt[o1] <= max[o1] && pt[o2] >= min[o2] && pt[o2] <= max[o2] {
            // Back t off 2 voxels if it's outside the chunk, to ensure we hit starting voxels.
            let t: OrderedFloat<f32> = (if t[mi] == 0.0 { 0.0 } else { t[mi] - 2.0 }).into();
            return Some(ChunkRayHit {
                chunk_coord,
                t,
                origin: (origin + direction * t.0) - chunk_coord.first_cell_coord().0.as_vec3(),
                direction,
            });
        }
    }

    // AABB test failed.
    return None;
}

fn raycast_chunk_voxels(chunk: &Chunk, chunk_ray_hit: ChunkRayHit) -> Option<VoxelRayHit> {
    let p = chunk_ray_hit.origin;
    let d = chunk_ray_hit.direction;
    let mut norm: Option<IVec3> = None;

    // How long we have traveled thus far (modified by initial 'jump to volume bounds').
    let mut t = f32::from(chunk_ray_hit.t);

    // Max distance we can travel. This is the corner to corner distance of the chunk, plus 2.
    let max_d = t + Vec3::splat(WIDTH as f32).length() + 2.0;

    // The starting voxel for the raycast.
    let mut i = p.floor().as_ivec3();

    // The directionVec we are stepping for each component.
    let step = d.signum().as_ivec3();

    // Just abs(Vec3::ONE / d) but accounts for zeros in the distance vector.
    let delta = (Vec3::new(
        if d.x.abs() < f32::EPSILON {
            f32::INFINITY
        } else {
            1.0 / d.x
        },
        if d.y.abs() < f32::EPSILON {
            f32::INFINITY
        } else {
            1.0 / d.y
        },
        if d.z.abs() < f32::EPSILON {
            f32::INFINITY
        } else {
            1.0 / d.z
        },
    ))
    .abs();

    let dist = Vec3::new(
        if step.x > 0 {
            i.x as f32 + 1.0 - p.x
        } else {
            p.x - i.x as f32
        },
        if step.y > 0 {
            i.y as f32 + 1.0 - p.y
        } else {
            p.y - i.y as f32
        },
        if step.z > 0 {
            i.z as f32 + 1.0 - p.z
        } else {
            p.z - i.z as f32
        },
    );

    // The nearest voxel boundary.
    let mut t_max = Vec3::new(
        if delta.x < f32::INFINITY {
            delta.x * dist.x
        } else {
            f32::INFINITY
        },
        if delta.y < f32::INFINITY {
            delta.y * dist.y
        } else {
            f32::INFINITY
        },
        if delta.z < f32::INFINITY {
            delta.z * dist.z
        } else {
            f32::INFINITY
        },
    );

    while t <= max_d {
        // Test if the current traverse is within the volume, and the voxel isn't empty.
        if i.cmpge(IVec3::ZERO).all()
            && i.cmplt(IVec3::splat(WIDTH as i32)).all()
            && chunk.get(LocalCoord(i.as_uvec3())) != default()
        {
            return Some(VoxelRayHit {
                world_coord: WorldCoord(chunk_ray_hit.chunk_coord.first_cell_coord().0 + i),
                distance: t,
                normal: norm.map(|n| n.into()),
            });
        }

        // Select the smallest t_max
        if t_max.x < t_max.y {
            if t_max.x < t_max.z {
                i.x += step.x;
                t = t_max.x;
                t_max.x += delta.x;
                norm = Some(IVec3::new(-step.x, 0, 0));
            } else {
                i.z += step.z;
                t = t_max.z;
                t_max.z += delta.z;
                norm = Some(IVec3::new(0, 0, -step.z));
            }
        } else {
            if t_max.y < t_max.z {
                i.y += step.y;
                t = t_max.y;
                t_max.y += delta.y;
                norm = Some(IVec3::new(0, -step.y, 0));
            } else {
                i.z += step.z;
                t = t_max.z;
                t_max.z += delta.z;
                norm = Some(IVec3::new(0, 0, -step.z));
            }
        }
    }

    return None;
}
