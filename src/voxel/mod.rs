mod buffer;
mod compressed_chunk;
mod coords;
mod tesselate;
mod volume;

pub use buffer::*;
pub use coords::*;
use serde::{Deserialize, Serialize};
pub use volume::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PbrProps {
    pub color: u32,
    pub metallic: u8,
    pub roughness: u8,
    pub reflectance: u8,
    pub emission: u32,
}

impl PbrProps {
    pub fn is_opaque(&self) -> bool {
        !(self.color & 0xFF000000) == 0
    }
}
