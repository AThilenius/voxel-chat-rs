use bevy::prelude::*;

mod buffer;
mod compressed_chunk;
mod coords;
mod mesh;
mod raycast;
mod volume;

pub use buffer::*;
pub use coords::*;
pub use mesh::*;
pub use raycast::*;
use serde::{Deserialize, Serialize};
pub use volume::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PbrProps {
    pub color: Rgba,
    pub metallic: u8,
    pub roughness: u8,
    pub reflectance: u8,
    pub emission: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Rgba {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Rgba {
    pub fn to_arr(&self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }

    pub fn shadow(&self, sa: bool) -> Self {
        if sa {
            let mut r = self.r as f32 / u8::MAX as f32;
            let mut g = self.g as f32 / u8::MAX as f32;
            let mut b = self.b as f32 / u8::MAX as f32;

            r = r / 2.0;
            g = g / 2.0;
            b = b / 2.0;

            Rgba {
                r: f32::round(r * u8::MAX as f32) as u8,
                g: f32::round(g * u8::MAX as f32) as u8,
                b: f32::round(b * u8::MAX as f32) as u8,
                a: self.a,
            }
        } else {
            return self.clone();
        }
    }
}

impl From<Rgba> for Color {
    fn from(rgba: Rgba) -> Self {
        Color::rgba_u8(rgba.r, rgba.g, rgba.b, rgba.a)
    }
}

impl From<Color> for Rgba {
    fn from(color: Color) -> Self {
        Rgba {
            r: (color.r() * 255.0) as u8,
            g: (color.g() * 255.0) as u8,
            b: (color.b() * 255.0) as u8,
            a: (color.a() * 255.0) as u8,
        }
    }
}
