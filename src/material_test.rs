use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::{MeshVertexAttribute, MeshVertexBufferLayout},
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
            VertexFormat,
        },
    },
};

pub struct MaterialTestPlugin;

impl Plugin for MaterialTestPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<VoxelMaterial>::default())
            .add_startup_system(setup);
    }
}

const ATTRIBUTE_METALLIC_ROUGHNESS: MeshVertexAttribute =
    MeshVertexAttribute::new("MetallicRoughness", 782560913, VertexFormat::Float32x2);
const ATTRIBUTE_EMISSION: MeshVertexAttribute =
    MeshVertexAttribute::new("Emission", 956190401, VertexFormat::Float32x4);
const ATTRIBUTE_REFLECTANCE: MeshVertexAttribute =
    MeshVertexAttribute::new("Reflectance", 582956105, VertexFormat::Float32);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<VoxelMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let color = Color::rgb(0.7, 0.1, 0.1);

    let mut mesh = Mesh::from(shape::Cube::default());
    mesh.generate_tangents().unwrap();

    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![color.as_linear_rgba_f32(); 24]);
    mesh.insert_attribute(ATTRIBUTE_METALLIC_ROUGHNESS, vec![[0.9, 0.2]; 24]);
    mesh.insert_attribute(ATTRIBUTE_EMISSION, vec![[0.0, 0.0, 0.0, 1.0]; 24]);
    mesh.insert_attribute(ATTRIBUTE_REFLECTANCE, vec![0.0; 24]);

    for x in 0..5 {
        for y in 0..5 {
            commands.spawn(MaterialMeshBundle {
                mesh: meshes.add(mesh.clone()),
                material: materials.add(VoxelMaterial {
                    normal_map_texture: asset_server.load("textures/normal_round.png"),
                }),
                transform: Transform::from_xyz(x as f32, y as f32 + 0.5, 0.0),
                ..default()
            });
        }
    }
}

#[derive(AsBindGroup, Debug, Clone, TypeUuid)]
#[uuid = "8cc0d9ab-e0ed-4a7d-b677-bbb8f0a00c41"]
pub struct VoxelMaterial {
    #[texture(1)]
    #[sampler(2)]
    normal_map_texture: Handle<Image>,
}

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
        let fragment = descriptor.fragment.as_mut().unwrap();
        fragment
            .shader_defs
            .push("STANDARDMATERIAL_NORMAL_MAP".into());

        let vertex_layout = layout.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
            Mesh::ATTRIBUTE_TANGENT.at_shader_location(3),
            Mesh::ATTRIBUTE_COLOR.at_shader_location(4),
            ATTRIBUTE_METALLIC_ROUGHNESS.at_shader_location(5),
            ATTRIBUTE_EMISSION.at_shader_location(6),
            ATTRIBUTE_REFLECTANCE.at_shader_location(7),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}
