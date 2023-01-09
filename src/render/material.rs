use crate::prelude::*;
use bevy::{
    asset::LoadState,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef},
};

#[derive(Resource)]
pub struct LoadingTexture {
    pub is_loaded: bool,
    pub handle: Handle<Image>,
}

pub fn load_chunk_texture(
    mut commands: Commands,
    mut server: ResMut<AssetServer>,
) {
    commands.insert_resource(LoadingTexture {
        is_loaded: false,
        handle: server.load("textures/array_test.png"),
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
    let array_layers = 3;
    image.reinterpret_stacked_2d_as_array(array_layers);
}

#[derive(AsBindGroup, Debug, Clone, TypeUuid)]
#[uuid = "9c5a0ddf-1eaf-41b4-9832-ed736fd26af3"]
pub struct ArrayTextureMaterial {
    #[texture(0, dimension = "2d_array")]
    #[sampler(1)]
    pub array_texture: Handle<Image>,
}

impl Material for ArrayTextureMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/array_texture.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Opaque
    }
}
