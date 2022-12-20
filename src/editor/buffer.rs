use crate::voxel::{ChunkCoord, LocalCoord, PbrProps, Volume, WorldCoord, COUNT};

/// A buffer of voxels, stored in chunks. Used exclusively by the editor.
#[derive(Default, Clone)]
pub struct Buffer {
    /// Chunks are Copy On Write, so cloning an entire buffer is cheap and mutations are only as
    /// expensive as the number of changes made.
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

        self.chunks
            .entry(chunk_coord)
            .or_insert_with(|| Default::default())
            .set(coord, cell);
    }

    pub fn count(&self) -> usize {
        self.chunks.values().map(|c| c.count).sum()
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

impl From<&Volume> for Buffer {
    fn from(volume: &Volume) -> Self {
        // Find the volume's AABB
    }
}

impl From<&Buffer> for Volume {
    fn from(buffer: &Buffer) -> Self {}
}
