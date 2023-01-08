use bevy::prelude::*;

use crate::voxel::{ChunkCoord, LocalCoord, PbrProps, WorldCoord, COUNT};

/// An arbitrarily sized buffer of voxels, stored in 32x32x32 chunks. Chunks are stored in an
/// immutable hashmap, meaning they are stored copy-on-write. This allows Buffers to be very cheaply
/// cloned and mutated.
///
/// Note on random-access performance:
/// Buffers are 'fast enough' for my use case, but each get/set incurs a hash lookup to find the
/// correct chunk. This can be reduced later if I need, but storing chunks in a contiguous array
/// with a lookup index separate.
#[derive(Default, Clone)]
pub struct Buffer {
    /// Chunks are Copy On Write, so cloning an entire buffer is cheap and mutations are only as
    /// expensive as the number of changes made. Chunks are GCed immediately when the non-default
    /// voxel count hits zero.
    pub chunks: im::HashMap<ChunkCoord, Chunk>,
}

// 32*32*32*11 bytes = 360 KB chunks
#[derive(Clone)]
pub struct Chunk {
    /// Voxels, linearized via `LocalCoord::linearize`.
    pub voxels: Vec<PbrProps>,

    /// Count of non-empty (all zero) voxels. Used for compacting.
    pub count: usize,
}

/// A facade to a Buffer for iterating voxels in world space. Reduces the number of hashmap lookups
/// by keeping a reference to the last accessed chunk.
pub struct FastBufferReader<'a> {
    buffer: &'a Buffer,
    chunk: Option<&'a Chunk>,
    chunk_coord: Option<ChunkCoord>,
}

impl Buffer {
    pub fn get<T>(&self, c: T) -> PbrProps
    where
        T: Into<WorldCoord>,
    {
        let coord: WorldCoord = c.into();
        let chunk_coord: ChunkCoord = coord.into();
        self.chunks
            .get(&chunk_coord)
            .map_or(Default::default(), |c| c.get(&coord))
    }

    pub fn set<T>(&mut self, c: T, cell: PbrProps)
    where
        T: Into<WorldCoord>,
    {
        let coord: WorldCoord = c.into();
        let chunk_coord: ChunkCoord = coord.into();

        let chunk = self
            .chunks
            .entry(chunk_coord)
            .or_insert_with(|| Default::default());

        chunk.set(coord, cell);

        if chunk.count == 0 {
            self.chunks.remove(&chunk_coord);
        }
    }

    pub fn count(&self) -> usize {
        self.chunks.values().map(|c| c.count).sum()
    }

    pub fn chunk_aabb(&self) -> (ChunkCoord, ChunkCoord) {
        let mut min = ChunkCoord(IVec3::new(std::i32::MAX, std::i32::MAX, std::i32::MAX));
        let mut max = ChunkCoord(IVec3::new(std::i32::MIN, std::i32::MIN, std::i32::MIN));

        for (coord, _) in self.chunks.iter() {
            min.0.x = min.0.x.min(coord.0.x);
            min.0.y = min.0.y.min(coord.0.y);
            min.0.z = min.0.z.min(coord.0.z);

            max.0.x = max.0.x.max(coord.0.x);
            max.0.y = max.0.y.max(coord.0.y);
            max.0.z = max.0.z.max(coord.0.z);
        }

        (min, max)
    }
}

impl Chunk {
    #[inline(always)]
    pub fn get<T>(&self, c: T) -> PbrProps
    where
        T: Into<LocalCoord>,
    {
        let local_coord = c.into();
        let idx = local_coord.linearize();
        self.voxels[idx]
    }

    #[inline(always)]
    pub fn set<T>(&mut self, c: T, mat: PbrProps)
    where
        T: Into<LocalCoord>,
    {
        let coord: LocalCoord = c.into();
        let idx = coord.linearize();

        // Track cell count.
        if self.voxels[idx] == Default::default() && mat != Default::default() {
            self.count += 1;
        } else if self.voxels[idx] != Default::default() && mat == Default::default() {
            self.count -= 1;
        }

        self.voxels[idx] = mat;
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            voxels: vec![Default::default(); COUNT],
            count: Default::default(),
        }
    }
}

impl<'a> FastBufferReader<'a> {
    pub fn new(buffer: &'a Buffer) -> Self {
        Self {
            buffer,
            chunk: None,
            chunk_coord: None,
        }
    }

    pub fn get<T>(&mut self, c: T) -> PbrProps
    where
        T: Into<WorldCoord>,
    {
        let coord: WorldCoord = c.into();
        let chunk_coord: ChunkCoord = coord.into();
        let local_coord: LocalCoord = coord.into();

        if self.chunk_coord != Some(chunk_coord) {
            self.chunk_coord = Some(chunk_coord);
            self.chunk = self.buffer.chunks.get(&chunk_coord);
        }

        self.chunk.map_or(default(), |c| c.get(local_coord))
    }
}
