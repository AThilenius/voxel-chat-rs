use bevy::prelude::*;

mod buffer;
mod coords;

use buffer::*;
use coords::*;

#[derive(Resource)]
pub struct EditorResource {
    /// The entity we are editing, if any.
    pub entity: Option<Entity>,
    pub buffer: Buffer,
    // TODO: I'm not sure this actually need to be Option<T>
    pub ephemeral_buffer: Option<Buffer>,
}
