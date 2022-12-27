#![allow(dead_code)]
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub const WIDTH: usize = 32;
pub const LN_SIZE: usize = 5;
pub const COUNT: usize = WIDTH * WIDTH * WIDTH;
const UPPER_MASK: i32 = !((WIDTH as i32) - 1);
const LOWER_MASK: usize = WIDTH - 1;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct WorldCoord(pub IVec3);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ChunkCoord(pub IVec3);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct LocalCoord(pub UVec3);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct WorldCoordOffset(pub IVec3);

#[derive(Serialize, Deserialize)]
pub enum Coord {
    Cell(IVec3),
    Chunk(IVec3),
    Local(UVec3),
}

impl From<(i32, i32, i32)> for WorldCoord {
    fn from(v: (i32, i32, i32)) -> Self {
        Self(IVec3::new(v.0, v.1, v.2))
    }
}

impl From<(i32, i32, i32)> for WorldCoordOffset {
    fn from(v: (i32, i32, i32)) -> Self {
        Self(IVec3::new(v.0, v.1, v.2))
    }
}

impl From<WorldCoord> for ChunkCoord {
    #[inline(always)]
    fn from(c: WorldCoord) -> Self {
        Self(IVec3::new(
            c.0.x >> LN_SIZE,
            c.0.y >> LN_SIZE,
            c.0.z >> LN_SIZE,
        ))
    }
}

impl From<&WorldCoord> for ChunkCoord {
    #[inline(always)]
    fn from(c: &WorldCoord) -> Self {
        Self(IVec3::new(
            c.0.x >> LN_SIZE,
            c.0.y >> LN_SIZE,
            c.0.z >> LN_SIZE,
        ))
    }
}

impl From<WorldCoord> for LocalCoord {
    #[inline(always)]
    fn from(c: WorldCoord) -> Self {
        Self(UVec3::new(
            (c.0.x - (c.0.x & UPPER_MASK)) as u32,
            (c.0.y - (c.0.y & UPPER_MASK)) as u32,
            (c.0.z - (c.0.z & UPPER_MASK)) as u32,
        ))
    }
}

impl From<&WorldCoord> for LocalCoord {
    #[inline(always)]
    fn from(c: &WorldCoord) -> Self {
        Self(UVec3::new(
            (c.0.x - (c.0.x & UPPER_MASK)) as u32,
            (c.0.y - (c.0.y & UPPER_MASK)) as u32,
            (c.0.z - (c.0.z & UPPER_MASK)) as u32,
        ))
    }
}

impl WorldCoord {
    #[inline(always)]
    pub fn from_offset_into_chunk(chunk_coord: &ChunkCoord, x: usize, y: usize, z: usize) -> Self {
        WorldCoord(IVec3::new(
            (chunk_coord.0.x << LN_SIZE) + x as i32,
            (chunk_coord.0.y << LN_SIZE) + y as i32,
            (chunk_coord.0.z << LN_SIZE) + z as i32,
        ))
    }

    fn iter(a: WorldCoord, b: WorldCoord) -> impl Iterator<Item = WorldCoord> {
        (a.0.z..=b.0.z)
            .flat_map(move |z| (a.0.y..=b.0.y).map(move |y| (y, z)))
            .flat_map(move |(y, z)| (a.0.x..=b.0.x).map(move |x| WorldCoord(IVec3::new(x, y, z))))
    }
}

impl ChunkCoord {
    pub fn first_cell_coord(&self) -> WorldCoord {
        WorldCoord(IVec3::new(
            self.0.x << LN_SIZE,
            self.0.y << LN_SIZE,
            self.0.z << LN_SIZE,
        ))
    }

    pub fn last_cell_coord(&self) -> WorldCoord {
        WorldCoord(IVec3::new(
            (self.0.x << LN_SIZE) + WIDTH as i32 - 1,
            (self.0.y << LN_SIZE) + WIDTH as i32 - 1,
            (self.0.z << LN_SIZE) + WIDTH as i32 - 1,
        ))
    }

    pub fn iter_world_coords(&self) -> impl Iterator<Item = WorldCoord> {
        let first = self.first_cell_coord();
        let last = self.last_cell_coord();
        WorldCoord::iter(first, last)
    }
}

impl LocalCoord {
    #[inline(always)]
    pub fn to_cell_coord(&self, chunk_coord: &ChunkCoord) -> WorldCoord {
        WorldCoord(self.0.as_ivec3() + chunk_coord.first_cell_coord().0)
    }

    #[inline(always)]
    pub fn linearize(&self) -> usize {
        (self.0.x as usize)
            + ((self.0.y as usize) << LN_SIZE)
            + (((self.0.z as usize) << LN_SIZE) << LN_SIZE)
    }
}

impl WorldCoordOffset {
    pub fn to_cell_coord(&self, anchor: WorldCoord) -> WorldCoord {
        WorldCoord(anchor.0 + self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_to_chunk_coord() {
        let c = WorldCoord(IVec3::new(0, 0, 0));
        assert_eq!(ChunkCoord::from(c), ChunkCoord(IVec3::new(0, 0, 0)));

        let c = WorldCoord(IVec3::new(-1, -1, -1));
        assert_eq!(ChunkCoord::from(c), ChunkCoord(IVec3::new(-1, -1, -1)));

        let c = WorldCoord(IVec3::new(31, 31, 31));
        assert_eq!(ChunkCoord::from(c), ChunkCoord(IVec3::new(0, 0, 0)));

        let c = WorldCoord(IVec3::new(32, 32, 32));
        assert_eq!(ChunkCoord::from(c), ChunkCoord(IVec3::new(1, 1, 1)));
    }

    #[test]
    fn test_chunk_coord() {
        let c = ChunkCoord(IVec3::new(0, 0, 0));
        assert_eq!(c.first_cell_coord(), WorldCoord(IVec3::new(0, 0, 0)));
        assert_eq!(c.last_cell_coord(), WorldCoord(IVec3::new(31, 31, 31)));

        let c = ChunkCoord(IVec3::new(-1, -1, -1));
        assert_eq!(c.first_cell_coord(), WorldCoord(IVec3::new(-32, -32, -32)));
        assert_eq!(c.last_cell_coord(), WorldCoord(IVec3::new(-1, -1, -1)));

        let c = ChunkCoord(IVec3::new(10, 10, 10));
        assert_eq!(c.first_cell_coord(), WorldCoord(IVec3::new(320, 320, 320)));
        assert_eq!(c.last_cell_coord(), WorldCoord(IVec3::new(351, 351, 351)));
    }

    #[test]
    fn test_local_coord() {
        let c = LocalCoord(UVec3::new(0, 0, 0));
        assert_eq!(c.linearize(), 0);

        let c = LocalCoord(UVec3::new(1, 0, 0));
        assert_eq!(c.linearize(), 1);

        let c = LocalCoord(UVec3::new(0, 1, 0));
        assert_eq!(c.linearize(), 32);

        let c = LocalCoord(UVec3::new(0, 0, 1));
        assert_eq!(c.linearize(), 1024);

        let c = LocalCoord(UVec3::new(1, 1, 1));
        assert_eq!(c.linearize(), 1057);
    }
}
