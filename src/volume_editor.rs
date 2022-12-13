use bevy::prelude::*;

// A lot of this structure can be copied from Logic Paint:
// - Volumes are ImmutableHashMap<ChunkCoord, Chunk>
// - The `overlay_volume` represents the currently rendering volume in it's entirety
//   - Each from it is cloned from the `base_volume` and the `painter` is used to apply the current
//     set of ops to the volume.
//   - The ImmutableHashMap will only clone chunks when needed then - nice and efficient.
// - Can have N 'storage buffers', including of course the mouse-follow, yank and clipboard buffers.
//   - `mouse_follow_volume` is applied to the `overlay_volume` each frame.

// Currently editing volume Entity is stored directly in the `VolumeEditor` Resource. Raycasts can
// then be made against a known Entity. Camera controls orbit the volume entity origin (voxel
// 0,0,0). Space is either world where the volume may be perceptibly rotated or local where the root
// of the entire model is inversely rotated to to align the current volume with camera axises.
//
// I need a name for... prefabs? I don't like 'scene'.
// Prefabs always have a single root entity. The user will never have control over the components in
// this root entity apart from adding a `Children`

pub struct VolumeEditorPlugin;

impl Plugin for VolumeEditorPlugin {
    fn build(&self, app: &mut App) {
        //
    }
}

enum PaintSpace {
    Volumetric,
}

enum State {
    Painting { space: PaintSpace, color: Color },
}

struct Volume;

#[derive(Resource)]
struct VolumeEditor {
    volume: Volume,
    root_entity: Entity,
    state: State,
    children: Children,
}
