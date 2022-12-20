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
}

impl LocalCoord {
    #[inline(always)]
    pub fn to_cell_coord(&self, chunk_coord: &ChunkCoord) -> WorldCoord {
        WorldCoord(self.0.as_ivec3() + chunk_coord.first_cell_coord().0)
    }

    #[inline(always)]
    pub fn linearize(&self) -> usize {
        (self.0.x as usize) + (self.0.y as usize)
            << LN_SIZE + (self.0.z as usize)
            << (LN_SIZE << LN_SIZE)
    }
}

impl WorldCoordOffset {
    pub fn to_cell_coord(&self, anchor: WorldCoord) -> WorldCoord {
        WorldCoord(anchor.0 + self.0)
    }
}
