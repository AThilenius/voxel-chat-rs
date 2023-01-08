use bevy::prelude::*;

use crate::voxel::{
    raycast_buffer_voxels, Buffer, PbrProps, Rgba, VoxelMaterial, VoxelRayHit, WorldCoord,
};

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_test)
            .add_system(editor_prep)
            .add_system(editor_primary_logic)
            .add_system(editor_finalize);
    }
}

// The editor focuses in on a specific "Prefab", which is just a root entity. That entity, or any
// one of it's children can be camera-focused to edit the voxel volume. That means that raycasts
// need to be translate into the global space of the camera-focus entity before they are traced.
//
// For simplicity we'll want a pipelines of systems. Extract from entities, Transform, then write
// back to entities.
//

#[derive(Resource)]
pub struct EditorResource {
    pub prefab_editor: Option<PrefabEditor>,
}

pub struct PrefabEditor {
    // The root prefab entity being edited.
    pub prefab_entity: Entity,
    pub entity: Entity,
    pub buffer: Buffer,
    pub ephemeral_buffer: Buffer,
    pub input: EditorInput,
}

#[derive(Default)]
pub struct EditorInput {
    pub input: Input<MouseButton>,
    pub world_ray: Ray,
    pub local_ray: Ray,
    pub ray_hit: Option<VoxelRayHit>,
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

    // buffer.set(WorldCoord((0, 0, 0).into()), p);
    // buffer.set(WorldCoord((1, 0, 0).into()), p);
    // buffer.set(WorldCoord((0, 0, 1).into()), p);
    // buffer.set(WorldCoord((0, 1, 0).into()), p);

    // Create a Transform, translated by 1 unit in the X direction and rotated 45 degrees around the
    // y axis.
    let mesh: Mesh = (&buffer).into();
    let entity = commands
        .spawn(MaterialMeshBundle {
            mesh: meshes.add(mesh.clone()),
            material: materials.add(VoxelMaterial {}),
            transform: Transform {
                translation: Vec3::new(1.0, 0.0, 0.0),
                rotation: Quat::from_axis_angle(Vec3::Y, 60.0),
                // scale: Vec3::ONE,
                ..default()
            },
            ..default()
        })
        .id();

    commands.insert_resource(EditorResource {
        prefab_editor: Some(PrefabEditor {
            prefab_entity: entity,
            entity,
            buffer: buffer.clone(),
            ephemeral_buffer: buffer.clone(),
            input: default(),
        }),
    });
}

fn editor_prep(
    mut voxel_editor: ResMut<EditorResource>,
    mouse_button_input: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    camera: Query<(&GlobalTransform, &Camera)>,
    global_transform: Query<&GlobalTransform>,
) {
    let prefab_editor = unwrap_or_return!(&mut voxel_editor.prefab_editor);
    let (transform, camera) = camera.single();
    let window = unwrap_or_return!(windows.get_primary());
    let cursor = unwrap_or_return!(window.cursor_position());
    let focus_transform = ok_or_return!(global_transform.get(prefab_editor.entity));

    // Create a ray from the camera at the cursor position, in the global space of the focused
    // entity.
    let world_ray = camera.viewport_to_world(transform, cursor).unwrap();

    // Transform the ray into the local space of focus_transform and cast it.
    if mouse_button_input.just_pressed(MouseButton::Left) {
        let transform = focus_transform.affine().inverse();
        let origin = transform.transform_point3(world_ray.origin);
        let direction = transform.transform_vector3(world_ray.direction).normalize();
        let local_ray = Ray { origin, direction };
        let ray_hit = raycast_buffer_voxels(&prefab_editor.buffer, local_ray);

        prefab_editor.input = EditorInput {
            input: mouse_button_input.clone(),
            world_ray,
            local_ray,
            ray_hit,
        };
    }
}

fn editor_primary_logic(mut voxel_editor: ResMut<EditorResource>) {
    let prefab_editor = unwrap_or_return!(&mut voxel_editor.prefab_editor);

    if prefab_editor.input.input.just_pressed(MouseButton::Left) {
        if let Some(hit) = prefab_editor.input.ray_hit {
            let coord = hit.world_coord.0 + hit.normal.unwrap_or_default();
            prefab_editor.buffer.set(
                WorldCoord(coord),
                PbrProps {
                    color: Rgba::from(Color::rgb(1.0, 0.0, 0.0)),
                    metallic: 3,
                    roughness: 23,
                    reflectance: 128,
                    emission: 0,
                },
            );
        }
    }
}

fn editor_finalize(
    mut voxel_editor: ResMut<EditorResource>,
    mut mesh_handles: Query<&mut Handle<Mesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let prefab_editor = unwrap_or_return!(&mut voxel_editor.prefab_editor);

    *mesh_handles.get_mut(prefab_editor.entity).unwrap() =
        meshes.add((&prefab_editor.buffer).into());
}
