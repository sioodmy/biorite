#![warn(clippy::disallowed_types)]
#![feature(async_closure)]
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
    window::PresentMode,
};
use bevy_embedded_assets::EmbeddedAssetPlugin;
use bevy_rapier3d::prelude::{RapierPhysicsPlugin, *};
use bevy_renet::RenetClientPlugin;
use clap::Parser;

mod auth;
mod camera;
mod config;
mod material;
mod menu;
mod mesh;
mod net;
mod raycast;
mod render;
mod state;

use config::Args;
use lazy_static::lazy_static;
use menu::MenuPlugin;
use render::RenderClientPlugin;
use state::GameState;

lazy_static! {
    pub static ref ARGS: Args = Args::parse();
}

fn main() {
    let args = config::Args::parse();

    App::new()
        .add_state::<GameState>()
        .insert_resource(args)
        .add_plugins(
            DefaultPlugins
                .build()
                .add_before::<bevy::asset::AssetPlugin, _>(EmbeddedAssetPlugin)
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
        .add_plugin(MenuPlugin)
        .add_plugin(NetworkClientPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        // .add_plugin(RapierDebugRenderPlugin::default())
        .run();
}
