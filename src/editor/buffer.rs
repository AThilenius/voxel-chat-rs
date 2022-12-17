use super::coords::{ChunkCoord, LocalCoord, WorldCoord, COUNT};

/// A buffer of voxels, stored in chunks. Used exclusively by the editor.
#[derive(Default, Clone)]
pub struct Buffer {
    /// Chunks are Copy On Write, so cloning an entire buffer is cheap and mutations are only as
    /// expensive as the number of changes made.
    pub chunks: im::HashMap<ChunkCoord, Chunk>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Mat {
    pub color: u32,
    pub metallic: u8,
    pub roughness: u8,
    pub reflectance: u8,
    pub emission: u32,
}

// 32*32*32*11 bytes = 360 KB chunks
#[derive(Clone)]
pub struct Chunk {
    /// Voxels, linearized via `LocalCoord::linearize`.
    pub voxels: Vec<Mat>,

    /// Count of non-empty (all zero) voxels. Used for compacting.
    pub count: usize,
}

impl Buffer {
    pub fn get<T>(&self, c: T) -> Mat
    where
        T: Into<WorldCoord>,
    {
        let coord: WorldCoord = c.into();
        let chunk_coord: ChunkCoord = coord.into();
        self.chunks
            .get(&chunk_coord)
            .map_or(Default::default(), |c| c.get(&coord))
    }

    pub fn set<T>(&mut self, c: T, cell: Mat)
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
    pub fn get<T>(&self, c: T) -> Mat
    where
        T: Into<LocalCoord>,
    {
        let local_coord = c.into();
        let idx = local_coord.linearize();
        self.voxels[idx]
    }

    #[inline(always)]
    pub fn set<T>(&mut self, c: T, mat: Mat)
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
