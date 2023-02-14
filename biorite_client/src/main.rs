#[cfg(not(target_os = "windows"))]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

use crate::net::NetworkClientPlugin;
use bevy::{
    prelude::*,
    render::{
        render_resource::*,
        settings::{Backends, WgpuSettings},
        texture::ImagePlugin,
    },
    window::{PresentMode, WindowDescriptor},
};
use bevy_embedded_assets::EmbeddedAssetPlugin;
use bevy_rapier3d::prelude::{RapierPhysicsPlugin, *};
use bevy_renet::RenetClientPlugin;

mod auth;
mod camera;
mod material;
mod mesh;
mod net;
mod raycast;
mod render;

use render::RenderClientPlugin;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    auth::send_public_key().await?;

    App::new()
        .insert_resource(WgpuSettings {
            backends: Some(Backends::VULKAN),
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(
            DefaultPlugins
                .build()
                .add_before::<bevy::asset::AssetPlugin, _>(EmbeddedAssetPlugin)
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        width: 1280.,
                        height: 720.,
                        transparent: false,
                        title: format!("Biorite {}", env!("CARGO_PKG_VERSION")),
                        resizable: true,
                        present_mode: PresentMode::AutoVsync,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .set(ImagePlugin {
                    default_sampler: SamplerDescriptor {
                        address_mode_u: AddressMode::Repeat,
                        address_mode_v: AddressMode::Repeat,
                        address_mode_w: AddressMode::Repeat,
                        mag_filter: FilterMode::Nearest,
                        min_filter: FilterMode::Nearest,
                        ..Default::default()
                    },
                }),
        )
        .add_plugin(RenetClientPlugin::default())
        .add_plugin(RenderClientPlugin)
        .add_plugin(NetworkClientPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        // .add_plugin(RapierDebugRenderPlugin::default())
        .run();

    Ok(())
}
