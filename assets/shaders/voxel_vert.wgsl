#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings
#import bevy_pbr::mesh_functions

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) pbr_norm: vec4<u32>,
    @location(2) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) color: vec4<f32>,
    @location(3) metallic: f32,
    @location(4) roughness: f32,
    @location(5) reflectance: f32,
    @location(6) emission: vec4<f32>,
};

var<private> NORMALS: array<vec3<f32>, 6> = array<vec3<f32>, 6>(
    vec3<f32>(-1., 0., 0.),
    vec3<f32>(0., 0., 1.),
    vec3<f32>(1., 0., 0.),
    vec3<f32>(0., 0., -1.),
    vec3<f32>(0., 1., 0.),
    vec3<f32>(0., -1., 0.),
);

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

    out.world_position = mesh_position_local_to_world(mesh.model, vec4<f32>(vertex.position.xyz, 1.0));
    out.clip_position = mesh_position_world_to_clip(out.world_position);

    var normal = NORMALS[vertex.pbr_norm.w];
    out.world_normal = mesh_normal_local_to_world(normal);
    out.color = vec4(vertex.color.rgb, 1.0);
    out.metallic = f32(vertex.pbr_norm.x) / 255.0;
    out.roughness = f32(vertex.pbr_norm.y) / 255.0;
    out.reflectance = f32(vertex.pbr_norm.z) / 255.0;

    var emission = vertex.color.a * 24.0;
    out.emission = vec4(vertex.color.rgb * emission, clamp(vertex.color.a, 0.0, 1.0));

    return out;
}
