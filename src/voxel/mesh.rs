use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::{Indices, MeshVertexAttribute, MeshVertexBufferLayout, VertexAttributeValues},
        render_resource::{
            AsBindGroup, PrimitiveTopology, RenderPipelineDescriptor, ShaderRef,
            SpecializedMeshPipelineError, VertexFormat,
        },
    },
};

use super::{Buffer, WorldCoord};

const ATTRIBUTE_COLOR_EMISSIVE: MeshVertexAttribute =
    MeshVertexAttribute::new("Vertex_Color_Emissive", 956190401, VertexFormat::Unorm8x4);

pub const ATTRIBUTE_PBR_NORM: MeshVertexAttribute =
    MeshVertexAttribute::new("Vertex_Pbr_Norm", 569923874, VertexFormat::Uint8x4);

/// Tuples of Quad Origin, Normal, Tangent, Bitangent
const NORM_TAN_BITAN: [(IVec3, IVec3, IVec3, IVec3); 6] = [
    (IVec3::ZERO, IVec3::NEG_X, IVec3::Z, IVec3::Y),
    (IVec3::Z, IVec3::Z, IVec3::X, IVec3::Y),
    (IVec3::new(1, 0, 1), IVec3::X, IVec3::NEG_Z, IVec3::Y),
    (IVec3::X, IVec3::NEG_Z, IVec3::NEG_X, IVec3::Y),
    (IVec3::Y, IVec3::Y, IVec3::Z, IVec3::X),
    (IVec3::ZERO, IVec3::NEG_Y, IVec3::X, IVec3::Z),
];

impl From<&Buffer> for Mesh {
    fn from(buffer: &Buffer) -> Self {
        let mut positions: Vec<IVec3> = Vec::new();
        let mut pbr_norm: Vec<[u8; 4]> = Vec::new();
        let mut color_emissive: Vec<[u8; 4]> = Vec::new();
        let mut indexes: Vec<u32> = Vec::new();

        // TODO: This is heinously inefficient, but IDGAF right now.
        for chunk_coord in buffer.chunks.keys() {
            for WorldCoord(coord) in chunk_coord.iter_world_coords() {
                let props = buffer.get(WorldCoord(coord));

                if props == default() {
                    continue;
                }

                for (i, (origin, norm, tan, bi_tan)) in NORM_TAN_BITAN.into_iter().enumerate() {
                    if buffer.get(WorldCoord(coord + norm)) != default() {
                        continue;
                    }

                    // Quad will be meshed.
                    let tan_shadow = buffer.get(WorldCoord(coord + norm + tan)) != default();
                    let neg_tan_shadow = buffer.get(WorldCoord(coord + norm - tan)) != default();
                    let bi_tan_shadow = buffer.get(WorldCoord(coord + norm + bi_tan)) != default();
                    let neg_bi_tan_shadow =
                        buffer.get(WorldCoord(coord + norm - bi_tan)) != default();

                    let ll_c = props.color.shadow(neg_tan_shadow, neg_bi_tan_shadow);
                    let lr_c = props.color.shadow(tan_shadow, neg_bi_tan_shadow);
                    let ur_c = props.color.shadow(tan_shadow, bi_tan_shadow);
                    let ul_c = props.color.shadow(neg_tan_shadow, bi_tan_shadow);

                    let p = coord + origin;

                    positions.extend([p, p + tan, p + tan + bi_tan, p + bi_tan]);
                    pbr_norm
                        .extend([[props.metallic, props.roughness, props.reflectance, i as u8]; 4]);
                    color_emissive.extend([
                        [ll_c.r, ll_c.g, ll_c.b, props.emission],
                        [lr_c.r, lr_c.g, lr_c.b, props.emission],
                        [ur_c.r, ur_c.g, ur_c.b, props.emission],
                        [ul_c.r, ul_c.g, ul_c.b, props.emission],
                    ]);
                    indexes.extend(
                        [0, 1, 2, 0, 2, 3]
                            .iter()
                            .map(|i| i + (positions.len() - 4) as u32),
                    );
                }
            }
        }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

        let use_u32_indexes = positions.len() > u16::MAX as usize;

        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            positions
                .into_iter()
                .map(|p| p.as_vec3().to_array())
                .collect::<Vec<_>>(),
        );

        mesh.insert_attribute(ATTRIBUTE_PBR_NORM, VertexAttributeValues::Uint8x4(pbr_norm));
        mesh.insert_attribute(
            ATTRIBUTE_COLOR_EMISSIVE,
            VertexAttributeValues::Unorm8x4(color_emissive),
        );

        if use_u32_indexes {
            mesh.set_indices(Some(Indices::U32(indexes)));
        } else {
            mesh.set_indices(Some(Indices::U16(
                indexes.into_iter().map(|i| i as u16).collect::<Vec<u16>>(),
            )));
        }

        mesh
    }
}

#[derive(AsBindGroup, Debug, Clone, TypeUuid)]
#[uuid = "8cc0d9ab-e0ed-4a7d-b677-bbb8f0a00c41"]
pub struct VoxelMaterial {}

impl Material for VoxelMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/voxel_vert.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/voxel_frag.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            ATTRIBUTE_PBR_NORM.at_shader_location(1),
            ATTRIBUTE_COLOR_EMISSIVE.at_shader_location(2),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}
