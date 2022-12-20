use bevy::prelude::*;

use crate::voxel::WorldCoordOffset;

use super::PbrProps;

/// A singe voxel volume, stored in RLN format.
pub struct Volume {
    pub origin_offset: WorldCoordOffset,
    pub runs: Vec<Run>,
    pub palette: Vec<PbrProps>,
}

pub struct Run {
    length: u32,
    palette_index: u32,
}
