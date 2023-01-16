mod constituents;
mod ui;

use bevy::prelude::*;

use crate::voxel::{Buffer, PbrProps, Rgba, VoxelMaterial, WorldCoord};

use self::{
    constituents::{gather_editor_constituents, EditorConstituents},
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
    pub commit_buffer: Buffer,
    pub buffer: Buffer,
    pub buffer_dirty: bool,
    pub material: PbrProps,
    pub undo_stack: Vec<Buffer>,
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

    // Create a Transform, translated by 1 unit in the X direction and rotated 45 degrees around the
    // y axis.
    let mesh: Mesh = (&buffer).into();
    let entity = commands
        .spawn(MaterialMeshBundle {
            mesh: meshes.add(mesh.clone()),
            material: materials.add(VoxelMaterial {}),
            transform: Transform {
                // translation: Vec3::new(1.0, 0.0, 0.0),
                // rotation: Quat::from_axis_angle(Vec3::Y, 60.0),
                scale: Vec3::splat(1.0 / 16.0),
                ..default()
            },
            ..default()
        })
        .id();

    commands.insert_resource(EditorResource {
        constituents: default(),
        prefab_entity: entity,
        entity,
        commit_buffer: buffer.clone(),
        buffer: buffer.clone(),
        buffer_dirty: false,
        material: p,
        undo_stack: default(),
    });
}

fn editor_primary_logic(
    mut voxel_editor: ResMut<EditorResource>,
    mut mesh_handles: Query<&mut Handle<Mesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let mouse = voxel_editor.constituents.mouse_buttons.clone();

    // Reset the buffer every frame. This is a very cheap operation because of COW semantics.
    voxel_editor.buffer = voxel_editor.commit_buffer.clone();

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
                voxel_editor.buffer_dirty = true;
                voxel_editor.buffer.set(world_coord, p);
            }
        }

        if mouse.just_released(MouseButton::Left) && voxel_editor.buffer_dirty {
            // Commit the buffer.
            let undo_buffer = voxel_editor.commit_buffer.clone();
            voxel_editor.undo_stack.push(undo_buffer);
            voxel_editor.commit_buffer = voxel_editor.buffer.clone();
            voxel_editor.buffer_dirty = false;
        }
    }

    // Handle undo (Ctrl + Z)
    if voxel_editor
        .constituents
        .keyboard
        .pressed(KeyCode::LControl)
        && voxel_editor.constituents.keyboard.just_pressed(KeyCode::Z)
    {
        if let Some(undo_buffer) = voxel_editor.undo_stack.pop() {
            voxel_editor.commit_buffer = undo_buffer.clone();
            voxel_editor.buffer = undo_buffer;
        }
    }

    // Finalize
    *mesh_handles.get_mut(voxel_editor.entity).unwrap() = meshes.add((&voxel_editor.buffer).into());
}
