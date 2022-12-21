use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::{Buffer, Chunk, ChunkCoord, PbrProps};

/// Analogous to a `Buffer` but stored chunk data in Run Length Encoded format.
///
/// TODO: This needs to be hardened against malicious input.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CompressedBuffer {
    pub chunks: HashMap<IVec3, CompressedChunk>,
}

/// Analogous to a `Chunk` but stores data in Run Length Encoded format.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CompressedChunk {
    runs: Vec<Run>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Run {
    pub len: u32,
    pub pbr_props: PbrProps,
}

impl From<&Chunk> for CompressedChunk {
    fn from(chunk: &Chunk) -> Self {
        // Convert the chunk into a vector of runs.
        let mut compressed_chunk = Self::default();
        let mut run = Run::default();

        for pbr_props in chunk.voxels.iter() {
            if run.pbr_props == *pbr_props {
                run.len += 1;
            } else {
                if run.len > 0 {
                    compressed_chunk.runs.push(run);
                }
                run = Run {
                    len: 1,
                    pbr_props: *pbr_props,
                };
            }
        }

        // Push the final run as well
        compressed_chunk.runs.push(run);
        compressed_chunk
    }
}

impl From<&Buffer> for CompressedBuffer {
    fn from(buffer: &Buffer) -> Self {
        Self {
            chunks: buffer
                .chunks
                .iter()
                .map(|(coord, chunk)| (coord.0, chunk.into()))
                .collect(),
        }
    }
}

impl From<&CompressedChunk> for Chunk {
    fn from(compressed_chunk: &CompressedChunk) -> Self {
        let mut chunk = Self::default();
        let mut i = 0;

        for run in &compressed_chunk.runs {
            if run.pbr_props != PbrProps::default() {
                chunk.count += run.len as usize;
            }

            for _ in 0..run.len {
                chunk.voxels[i] = run.pbr_props;
                i += 1;
            }
        }

        chunk
    }
}

impl From<&CompressedBuffer> for Buffer {
    fn from(compressed_buffer: &CompressedBuffer) -> Self {
        Self {
            chunks: compressed_buffer
                .chunks
                .iter()
                .map(|(coord, compressed_chunk)| (ChunkCoord(*coord), compressed_chunk.into()))
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::voxel::WorldCoord;

    use super::*;

    #[test]
    fn test_compression() {
        let mut buffer = Buffer::default();
        let p = PbrProps {
            metallic: 255,
            ..Default::default()
        };

        buffer.set(WorldCoord(IVec3::new(0, 0, 0)), p);
        buffer.set(WorldCoord(IVec3::new(1, 0, 0)), p);
        buffer.set(WorldCoord(IVec3::new(-1, 0, 0)), p);
        buffer.set(WorldCoord(IVec3::new(40, 40, 40)), p);

        let compressed_buffer = CompressedBuffer::from(&buffer);
        let buffer = Buffer::from(&compressed_buffer);

        assert_eq!(buffer.count(), 4);
        assert_eq!(buffer.get(WorldCoord(IVec3::new(0, 0, 0))), p);
        assert_eq!(buffer.get(WorldCoord(IVec3::new(1, 0, 0))), p);
        assert_eq!(buffer.get(WorldCoord(IVec3::new(-1, 0, 0))), p);
        assert_eq!(buffer.get(WorldCoord(IVec3::new(40, 40, 40))), p);
    }
}
