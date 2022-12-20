mod coords;
mod volume;

pub use coords::*;
pub use volume::*;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PbrProps {
    pub color: u32,
    pub metallic: u8,
    pub roughness: u8,
    pub reflectance: u8,
    pub emission: u32,
}
