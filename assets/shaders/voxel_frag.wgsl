#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings
#import bevy_pbr::mesh_functions

@group(1) @binding(1)
var normal_map_texture: texture_2d<f32>;
@group(1) @binding(2)
var normal_map_sampler: sampler;

#import bevy_pbr::pbr_types
#import bevy_pbr::utils
#import bevy_pbr::clustered_forward
#import bevy_pbr::lighting
#import bevy_pbr::shadows
#import bevy_pbr::pbr_functions

struct FragmentInput {
    @builtin(front_facing) is_front: bool,
    @builtin(position) frag_coord: vec4<f32>,

    // Locations [0-4]
    #import bevy_pbr::mesh_vertex_output

    @location(5) metallic_roughness: vec2<f32>,
    @location(6) emission: vec4<f32>,
    @location(7) reflectance: f32,
};

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
        (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u,
        in.is_front,
    );

    pbr_input.is_orthographic = view.projection[3].w == 1.0;

    pbr_input.N = apply_normal_mapping(
        pbr_input.material.flags,
        pbr_input.world_normal,
        in.world_tangent,
        vec2<f32>(in.uv.x, 1.0 - in.uv.y),
    );

    pbr_input.V = calculate_view(in.world_position, pbr_input.is_orthographic);

    var output_color = pbr(pbr_input);

    #ifdef TONEMAP_IN_SHADER
        output_color = tone_mapping(output_color);
    #endif

    return output_color;
}
