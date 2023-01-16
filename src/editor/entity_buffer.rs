use crate::voxel::Buffer;

pub use bevy::prelude::*;

#[derive(Component, Default)]
pub struct EntityBuffer {
    pub buffer_dirty: bool,
    pub buffer: Buffer,
    pub commit_buffer: Buffer,
    pub undo_stack: Vec<Buffer>,
}
