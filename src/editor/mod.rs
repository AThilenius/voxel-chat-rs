mod constituents;
mod entity_buffer;
mod ui;

use bevy::prelude::*;

use crate::voxel::{Buffer, PbrProps, Rgba, VoxelMaterial, WorldCoord};

use self::{
    constituents::{gather_editor_constituents, EditorConstituents},
    entity_buffer::EntityBuffer,
    ui::editor_ui,
};

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_test)
            .add_system(gather_editor_constituents)
            .add_system(editor_ui.after(gather_editor_constituents))
            .add_system(editor_primary_logic.after(gather_editor_constituents));
    }
}

#[derive(Resource)]
pub struct EditorResource {
    pub constituents: EditorConstituents,
    pub prefab_entity: Entity,
    pub entity: Entity,
    pub material: PbrProps,
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

    // Set all cube alone the outline of the volume (0, 0, 0) to (31, 31, 31)
    for x in 0..32 {
        buffer.set(WorldCoord((x, 0, 0).into()), p);
        buffer.set(WorldCoord((x, 31, 0).into()), p);
        buffer.set(WorldCoord((x, 0, 31).into()), p);
        buffer.set(WorldCoord((x, 31, 31).into()), p);

        buffer.set(WorldCoord((0, x, 0).into()), p);
        buffer.set(WorldCoord((31, x, 0).into()), p);
        buffer.set(WorldCoord((0, x, 31).into()), p);
        buffer.set(WorldCoord((31, x, 31).into()), p);

        buffer.set(WorldCoord((0, 0, x).into()), p);
        buffer.set(WorldCoord((31, 0, x).into()), p);
        buffer.set(WorldCoord((0, 31, x).into()), p);
        buffer.set(WorldCoord((31, 31, x).into()), p);
    }

    let mesh: Mesh = (&buffer).into();

    // Spawn an entity tree with names
    let entity = commands
        .spawn((Name::from("Root"), SpatialBundle::default()))
        .id();

    let child_1 = commands
        .spawn((
            Name::from("Child 1"),
            MaterialMeshBundle {
                mesh: meshes.add(mesh.clone()),
                material: materials.add(VoxelMaterial {}),
                transform: Transform::from_translation(Vec3::new(-17.0, 0.0, 0.0)),
                ..default()
            },
            EntityBuffer {
                commit_buffer: buffer.clone(),
                buffer: buffer.clone(),
                buffer_dirty: false,
                undo_stack: default(),
            },
        ))
        .id();

    let child_2 = commands
        .spawn((
            Name::from("Child 2"),
            MaterialMeshBundle {
                mesh: meshes.add(mesh.clone()),
                material: materials.add(VoxelMaterial {}),
                transform: Transform {
                    translation: Vec3::new(17.0, 0.0, 0.0),
                    rotation: Quat::from_euler(EulerRot::XYZ, 1.0, 1.0, 1.0),
                    ..default()
                },
                ..default()
            },
            EntityBuffer {
                commit_buffer: buffer.clone(),
                buffer: buffer.clone(),
                buffer_dirty: false,
                undo_stack: default(),
            },
        ))
        .id();

    commands.entity(entity).push_children(&[child_1, child_2]);

    commands.insert_resource(EditorResource {
        constituents: default(),
        prefab_entity: entity,
        entity: child_1,
        material: p,
    });
}

fn editor_primary_logic(
    mut voxel_editor: ResMut<EditorResource>,
    mut mesh_handles: Query<&mut Handle<Mesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut entity_buffers: Query<&mut EntityBuffer>,
) {
    let mouse = voxel_editor.constituents.mouse_buttons.clone();
    let mut entity_buffer = entity_buffers.get_mut(voxel_editor.entity).unwrap();

    // Reset the buffer every frame. This is a very cheap operation because of COW semantics.
    entity_buffer.buffer = entity_buffer.commit_buffer.clone();

    if let (Some(drag_origin), Some(ray_hit)) = (
        voxel_editor.constituents.drag_origin.clone(),
        voxel_editor.constituents.ray_hit.clone(),
    ) {
        // Completely redraw the buffer if either left is down, or if it was just released.
        if mouse.pressed(MouseButton::Left) || mouse.just_released(MouseButton::Left) {
            let del = voxel_editor
                .constituents
                .keyboard
                .pressed(KeyCode::LControl);

            let p = if del {
                default()
            } else {
                voxel_editor.material
            };

            let start = drag_origin.voxel;
            let end = WorldCoord(ray_hit.world_coord.0 + ray_hit.normal.unwrap_or_default());
            for world_coord in WorldCoord::iter_range(start, end) {
                entity_buffer.buffer_dirty = true;
                entity_buffer.buffer.set(world_coord, p);
            }
        }

        if mouse.just_released(MouseButton::Left) && entity_buffer.buffer_dirty {
            // Commit the buffer.
            let undo_buffer = entity_buffer.commit_buffer.clone();
            entity_buffer.undo_stack.push(undo_buffer);
            entity_buffer.commit_buffer = entity_buffer.buffer.clone();
            entity_buffer.buffer_dirty = false;
        }
    }

    // Handle undo (Ctrl + Z)
    if voxel_editor
        .constituents
        .keyboard
        .pressed(KeyCode::LControl)
        && voxel_editor.constituents.keyboard.just_pressed(KeyCode::Z)
    {
        if let Some(undo_buffer) = entity_buffer.undo_stack.pop() {
            entity_buffer.commit_buffer = undo_buffer.clone();
            entity_buffer.buffer = undo_buffer;
        }
    }

    // Finalize
    *mesh_handles.get_mut(voxel_editor.entity).unwrap() =
        meshes.add((&entity_buffer.buffer).into());
}
