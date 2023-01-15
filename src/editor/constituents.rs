use bevy::prelude::*;

use crate::voxel::{raycast_buffer_voxels, VoxelRayHit, WorldCoord};

use super::EditorResource;

/// All the necessary 'input' state for the editor to function, including pre-processed rays, inputs
/// and so on.
#[derive(Default)]
pub struct EditorConstituents {
    pub prefab_global_transform: GlobalTransform,
    pub focus_global_transform: GlobalTransform,
    pub camera_global_transform: GlobalTransform,
    pub camera: Camera,
    pub world_ray: Ray,
    pub local_ray: Ray,
    pub ray_hit: Option<VoxelRayHit>,
    pub cursor: Vec2,
    pub drag_origin: Option<DragOrigin>,
    pub mouse_buttons: Input<MouseButton>,
}

#[derive(Clone)]
pub struct DragOrigin {
    pub voxel: WorldCoord,
    pub world_ray: Ray,
    pub local_ray: Ray,
    pub ray_hit: VoxelRayHit,
    pub mouse_buttons: Input<MouseButton>,
}

pub fn gather_editor_constituents(
    mut voxel_editor: ResMut<EditorResource>,
    camera: Query<(&GlobalTransform, &Camera)>,
    mouse_button_input: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    global_transforms: Query<&GlobalTransform>,
) {
    // Gather constituents
    voxel_editor.constituents = {
        let window = unwrap_or_return!(windows.get_primary());
        let cursor = unwrap_or_return!(window.cursor_position());
        let prefab_global_transform =
            ok_or_return!(global_transforms.get(voxel_editor.prefab_entity));
        let focus_global_transform = ok_or_return!(global_transforms.get(voxel_editor.entity));
        let (camera_global_transform, camera) = camera.single();

        // Create a ray from the camera at the cursor position, in the global space of the focused
        // entity.
        let world_ray = camera
            .viewport_to_world(camera_global_transform, cursor)
            .unwrap();

        // Transform the ray into the local space of focus_transform and cast it.
        let local_ray = {
            let transform = focus_global_transform.affine().inverse();
            let origin = transform.transform_point3(world_ray.origin);
            let direction = transform.transform_vector3(world_ray.direction).normalize();
            Ray { origin, direction }
        };

        let ray_hit = raycast_buffer_voxels(&voxel_editor.commit_buffer, local_ray);

        let drag_origin = {
            if mouse_button_input.just_pressed(MouseButton::Left)
                || mouse_button_input.just_pressed(MouseButton::Right)
            {
                if let Some(ray_hit) = ray_hit {
                    Some(DragOrigin {
                        voxel: ray_hit.world_coord,
                        world_ray,
                        local_ray,
                        ray_hit,
                        mouse_buttons: mouse_button_input.clone(),
                    })
                } else {
                    None
                }
            } else {
                // Otherwise don't change it.
                voxel_editor.constituents.drag_origin.clone()
            }
        };

        EditorConstituents {
            prefab_global_transform: *prefab_global_transform,
            camera_global_transform: *camera_global_transform,
            focus_global_transform: *focus_global_transform,
            camera: camera.clone(),
            world_ray,
            local_ray,
            ray_hit,
            cursor,
            drag_origin,
            mouse_buttons: mouse_button_input.clone(),
        }
    };
}
