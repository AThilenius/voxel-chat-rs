#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings

// @group(1) @binding(0)
// var<uniform> mesh: Mesh;

@group(1) @binding(1)
var normal_map_texture: texture_2d<f32>;
@group(1) @binding(2)
var normal_map_sampler: sampler;

#import bevy_pbr::mesh_functions
#import bevy_pbr::pbr_types
#import bevy_pbr::utils
#import bevy_pbr::clustered_forward
#import bevy_pbr::lighting
#import bevy_pbr::shadows
#import bevy_pbr::pbr_functions

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

struct FragmentInput {
    @builtin(front_facing) is_front: bool,
    @builtin(position) frag_coord: vec4<f32>,

    // Locations [0-4]
    #import bevy_pbr::mesh_vertex_output

    @location(5) metallic_roughness: vec2<f32>,
    @location(6) emission: vec4<f32>,
    @location(7) reflectance: f32,
};

fn custom_apply_normal_mapping(
    world_normal: vec3<f32>,
    world_tangent: vec4<f32>,
    uv: vec2<f32>,
) -> vec3<f32> {
    var N: vec3<f32> = world_normal;
    var T: vec3<f32> = world_tangent.xyz;
    var B: vec3<f32> = world_tangent.w * cross(N, T);

    // Nt is the tangent-space normal.
    var Nt = textureSample(normal_map_texture, normal_map_sampler, uv).rgb;
    Nt = Nt * 2.0 - 1.0;
    N = Nt.x * T + Nt.y * B + Nt.z * N;

    return normalize(N);
}

// See: https://github.com/bevyengine/bevy/blob/main/crates/bevy_pbr/src/render/pbr.wgsl
@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    var pbr_input: PbrInput = pbr_input_new();

    pbr_input.material.base_color = in.color;
    pbr_input.material.reflectance = in.reflectance;
    pbr_input.material.metallic = in.metallic_roughness.r;
    pbr_input.material.perceptual_roughness = in.metallic_roughness.g;
    pbr_input.material.emissive = in.emission;

    pbr_input.frag_coord = in.frag_coord;
    pbr_input.world_position = in.world_position;
    pbr_input.world_normal = prepare_world_normal(
        in.world_normal,
        false, // Double-sided, originally from flags
        in.is_front,
    );

    pbr_input.is_orthographic = view.projection[3].w == 1.0;

    // pbr_input.N = apply_normal_mapping(
    //     pbr_input.material.flags,
    //     pbr_input.world_normal,
    //     in.world_tangent,
    //     in.uv,
    // );

    pbr_input.N = custom_apply_normal_mapping(
        pbr_input.world_normal,
        in.world_tangent,
        in.uv,
    );

    pbr_input.V = calculate_view(in.world_position, pbr_input.is_orthographic);
    var output_color = pbr(pbr_input);

// #ifdef TONEMAP_IN_SHADER
//     output_color = tone_mapping(output_color);
// #endif

// #ifdef DEBAND_DITHER
//     var output_rgb = output_color.rgb;
//     output_rgb = pow(output_rgb, vec3<f32>(1.0 / 2.2));
//     output_rgb = output_rgb + screen_space_dither(in.frag_coord.xy);
//     output_rgb = pow(output_rgb, vec3<f32>(2.2));
//     output_color = vec4(output_rgb, output_color.a);
// #endif

    return output_color;
}
