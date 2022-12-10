use voxel_chat::core_main;

// Thinking place...
// Let's start with voxel editing and world-building.
// - Initial voxel volume is stored roughly: RLN of 24-bit color components. Just use MsgPack for
//   this. No PBR attributes yet (it requires some heavy Bevy rendering work).
// - For MVP here, there are no animations or scripts. Just voxel data, rotation and translation.
// - Scenes can be transitive, no? Each entity encodes any set of entities under it, each with their
//   own translation, rotation and voxel data.
// - Voxel data is stored content-addressed, Blake2 and is referenced by a URI: VC/VOLUME/<HASH>
// - Because MVP is web, this requires basic networking, and remote asset saving / fetching
//   - Don't get fancy with this part. Throw assets into Sled on the server and request things
//     as-needed one at a time (no 'bundles' yet).

// ## Volume Loading
// Play with the "AssetServer" to solve voxel loading and compilation into Mesh/Material.
// Code will look something like...
// let (mesh, material, volume) = assets.load(volume_blake2_uri)
//
// This will go over the network to fetch the asset in RLN format, decompress it and tesselate it.
// I don't think any systems or components need to be involved here.
// Alternative APIs can be something like:
// commands.spawn(PbrVolumeBundle::from_blake2_uri(uri));
// Only question then is what it looks like when you serialize the scene.

// ## Volume Editor
// The volume editor doesn't need to have anything at all to do with compiled volume assets (and
// probably shouldn't). Start by just throwing voxels into a HashMap and see what kind of
// performance we can get. Hopefully 'good enough'. Otherwise maybe look into that OctTree impl.
//
//
// enum EditorState
// Resource<VolumeEditorData>

fn main() {
    core_main();
}
