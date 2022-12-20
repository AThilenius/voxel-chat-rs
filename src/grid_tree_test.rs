use bevy::prelude::*;

pub struct GridTreeTestPlugin;

impl Plugin for GridTreeTestPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup);
    }
}

pub fn setup(// mut commands: Commands,
) {
    trace!("Hello")
}
