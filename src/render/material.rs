use crate::prelude::*;

use bevy::{
    asset::LoadState,
    pbr::{MaterialPipeline, MaterialPipelineKey},
    reflect::TypeUuid,
    render::{
        mesh::{MeshVertexAttribute, MeshVertexBufferLayout},
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef,
            SpecializedMeshPipelineError, VertexFormat,
        },
    },
};

#[derive(Resource)]
pub struct LoadingTexture {
    pub is_loaded: bool,
    pub handle: Handle<Image>,
    pub material: Handle<ArrayTextureMaterial>,
}

pub fn load_chunk_texture(
    mut commands: Commands,
    server: ResMut<AssetServer>,
    mut materials: ResMut<Assets<ArrayTextureMaterial>>,
) {
    let texture = server.load("textures/array_test.png");
    commands.insert_resource(LoadingTexture {
        is_loaded: false,
        handle: texture.clone(),
        material: materials.add(ArrayTextureMaterial {
            array_texture: texture,
        }),
    });
}

pub fn create_array_texture(
    asset_server: Res<AssetServer>,
    mut loading_texture: ResMut<LoadingTexture>,
    mut images: ResMut<Assets<Image>>,
) {
    if loading_texture.is_loaded
        || asset_server.get_load_state(loading_texture.handle.clone())
            != LoadState::Loaded
    {
        return;
    }
    loading_texture.is_loaded = true;
    let image = images.get_mut(&loading_texture.handle).unwrap();

    // Create a new array texture asset from the loaded texture.
    let array_layers = 5;
    image.reinterpret_stacked_2d_as_array(array_layers);
}

#[derive(AsBindGroup, Debug, Clone, TypeUuid)]
#[uuid = "9c5a0ddf-1eaf-41b4-9832-ed736fd26af3"]
pub struct ArrayTextureMaterial {
    #[texture(0, dimension = "2d_array")]
    #[sampler(1)]
    pub array_texture: Handle<Image>,
}

impl ArrayTextureMaterial {
    pub const ATTRIBUTE_TEXTURE_INDEX: MeshVertexAttribute =
        MeshVertexAttribute::new("index", 2137, VertexFormat::Sint32);
    pub const ATTRIBUTE_LIGHT: MeshVertexAttribute =
        MeshVertexAttribute::new("light", 2138, VertexFormat::Float32);
}
impl Material for ArrayTextureMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/array_texture.wgsl".into()
    }
    fn fragment_shader() -> ShaderRef {
        "shaders/array_texture.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Opaque
    }
    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
            ArrayTextureMaterial::ATTRIBUTE_TEXTURE_INDEX.at_shader_location(3),
            ArrayTextureMaterial::ATTRIBUTE_LIGHT.at_shader_location(4),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        debug!("{:?}", descriptor.vertex.buffers);
        Ok(())
    }
}
