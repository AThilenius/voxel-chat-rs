use bevy::prelude::*;

use crate::voxel::{raycast_buffer_voxels, Buffer, PbrProps, Rgba, VoxelMaterial, WorldCoord};

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_test)
            .add_system(raycast_at_mouse_click);
    }
}

#[derive(Resource)]
pub struct EditorResource {
    /// The entity we are editing, if any.
    pub entity: Option<Entity>,
    pub buffer: Buffer,
    // TODO: I'm not sure this actually need to be Option<T>
    // pub ephemeral_buffer: Option<Buffer>,
}

fn setup_test(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<VoxelMaterial>>,
) {
    let mut buffer = Buffer::default();
    let p = PbrProps {
        color: Rgba::from(Color::rgb(1.0, 0.0, 0.0)),
        metallic: 3,
        roughness: 23,
        reflectance: 128,
        emission: 0,
    };

    buffer.set(WorldCoord((0, 0, 0).into()), p);
    buffer.set(WorldCoord((1, 0, 0).into()), p);
    buffer.set(WorldCoord((0, 0, 1).into()), p);
    buffer.set(WorldCoord((0, 1, 0).into()), p);

    let mesh: Mesh = (&buffer).into();
    let entity = commands
        .spawn(MaterialMeshBundle {
            mesh: meshes.add(mesh.clone()),
            material: materials.add(VoxelMaterial {}),
            transform: Transform::IDENTITY,
            ..default()
        })
        .id();

    commands.insert_resource(EditorResource {
        entity: Some(entity),
        buffer,
    });
}

fn raycast_at_mouse_click(
    mut voxel_editor: ResMut<EditorResource>,
    mouse_button_input: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    camera: Query<(&GlobalTransform, &Camera)>,
    mut mesh_handles: Query<(&mut Handle<Mesh>)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let (transform, camera) = camera.single();
    let window = unwrap_or_return!(windows.get_primary());
    let cursor = unwrap_or_return!(window.cursor_position());

    let left = mouse_button_input.just_pressed(MouseButton::Left);

    if left {
        let ray = camera.viewport_to_world(transform, cursor).unwrap();

        if let Some(hit) = raycast_buffer_voxels(&voxel_editor.buffer, ray) {
            let coord = hit.world_coord.0 + hit.normal.unwrap_or_default();
            voxel_editor.buffer.set(
                WorldCoord(coord),
                PbrProps {
                    color: Rgba::from(Color::rgb(0.0, 1.0, 0.0)),
                    metallic: 3,
                    roughness: 23,
                    reflectance: 128,
                    emission: 0,
                },
            );

            *mesh_handles.get_mut(voxel_editor.entity.unwrap()).unwrap() =
                meshes.add((&voxel_editor.buffer).into());
        }
    }
}
