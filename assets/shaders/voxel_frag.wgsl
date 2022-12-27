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

    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) color: vec4<f32>,
    @location(3) metallic: f32,
    @location(4) roughness: f32,
    @location(5) reflectance: f32,
    @location(6) emission: vec4<f32>,
};

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    var pbr_input: PbrInput = pbr_input_new();

    pbr_input.frag_coord = in.frag_coord;
    pbr_input.material.base_color = in.color;
    pbr_input.material.metallic = in.metallic;
    pbr_input.material.perceptual_roughness = in.roughness;
    pbr_input.material.reflectance = in.reflectance;
    pbr_input.material.emissive = in.emission;

    pbr_input.world_position = in.world_position;
    pbr_input.is_orthographic = view.projection[3].w == 1.0;
    pbr_input.world_normal = in.world_normal;
    pbr_input.N = apply_normal_mapping(
        pbr_input.material.flags,
        pbr_input.world_normal
    );
    pbr_input.V = calculate_view(in.world_position, pbr_input.is_orthographic);

    var output_color = pbr(pbr_input);
    #ifdef TONEMAP_IN_SHADER
        output_color = tone_mapping(output_color);
    #endif

    return output_color;
}
