use bevy::{
    asset::LoadState,
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::MeshVertexBufferLayout,
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
        },
    },
};

pub struct MaterialTestPlugin;

impl Plugin for MaterialTestPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(MaterialPlugin::<ArrayTextureMaterial>::default())
            .add_startup_system(setup)
            .add_system(create_array_texture);
    }
}

#[derive(Resource)]
struct LoadingTexture {
    is_loaded: bool,
    handle: Handle<Image>,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Start loading the texture.
    // commands.insert_resource(LoadingTexture {
    //     is_loaded: false,
    //     handle: asset_server.load("textures/array_texture.png"),
    // });
}

fn create_array_texture(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    // mut loading_texture: ResMut<LoadingTexture>,
    // mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ArrayTextureMaterial>>,
) {
    let mut render_mesh = Mesh::new(PrimitiveTopology::TriangleList);
    render_mesh.insert_attribute(
        Mesh::,
        VertexAttributeValues::Float32x3(positions),
    );




    // if loading_texture.is_loaded
    //     || asset_server.get_load_state(loading_texture.handle.clone()) != LoadState::Loaded
    // {
    //     return;
    // }
    // loading_texture.is_loaded = true;
    // let image = images.get_mut(&loading_texture.handle).unwrap();

    // Create a new array texture asset from the loaded texture.
    // let array_layers = 4;
    // image.reinterpret_stacked_2d_as_array(array_layers);

    // Spawn some cubes using the array texture
    let mesh_handle = meshes.add(Mesh::from(shape::Icosphere {
        radius: 1.0,
        ..default()
    }));
    let material_handle = materials.add(ArrayTextureMaterial {
        normal_texture: asset_server.load("textures/normal.png"),
    });
    for x in -5..=5 {
        commands.spawn(MaterialMeshBundle {
            mesh: mesh_handle.clone(),
            material: material_handle.clone(),
            transform: Transform::from_xyz(x as f32 + 0.5, 0.0, -2.0),
            ..Default::default()
        });
    }
}

#[derive(AsBindGroup, Debug, Clone, TypeUuid)]
#[uuid = "9c5a0ddf-1eaf-41b4-9832-ed736fd26af3"]
struct ArrayTextureMaterial {
    #[texture(0, dimension = "2d")]
    #[sampler(1)]
    normal_texture: Handle<Image>,
}

impl Material for ArrayTextureMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/voxel.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor
            .fragment
            .as_mut()
            .unwrap()
            .shader_defs
            .push("STANDARDMATERIAL_NORMAL_MAP".into());
        Ok(())
    }
}
