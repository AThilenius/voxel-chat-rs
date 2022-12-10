#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings
#import bevy_pbr::mesh_functions

// See: https://github.com/bevyengine/bevy/blob/main/crates/bevy_pbr/src/render/mesh.wgsl
struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) tangent: vec4<f32>,
    @location(4) color: vec4<f32>,
    @location(5) metallic_roughness: vec2<f32>,
    @location(6) emission: vec4<f32>,
    @location(7) reflectance: f32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,

    // Locations [0-4]
    #import bevy_pbr::mesh_vertex_output

    @location(5) metallic_roughness: vec2<f32>,
    @location(6) emission: vec4<f32>,
    @location(7) reflectance: f32,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

    out.world_position = mesh_position_local_to_world(mesh.model, vec4<f32>(vertex.position, 1.0));
    out.clip_position = mesh_position_world_to_clip(out.world_position);
    out.world_normal = mesh_normal_local_to_world(vertex.normal);
    out.uv = vertex.uv;
    out.world_tangent = mesh_tangent_local_to_world(mesh.model, vertex.tangent);
    out.color = vertex.color;
    out.metallic_roughness = vertex.metallic_roughness;
    out.emission = vertex.emission;
    out.reflectance = vertex.reflectance;

    return out;
}
