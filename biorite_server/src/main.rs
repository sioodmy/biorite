#[cfg(not(target_os = "windows"))]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

use crate::net::NetworkServerPlugin;
use bevy::{log::LogPlugin, prelude::*, render::settings::WgpuSettings};
use bevy_rapier3d::prelude::*;
use biorite_generator::SaveFile;

pub mod block_update;
mod chunks;
mod collider;
mod net;

fn main() {
    App::new()
        .insert_resource(WgpuSettings {
            backends: None,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins.set(LogPlugin {
            level: bevy::log::Level::ERROR,
            filter: "error,wgpu_core=warn,wgpu_hal=warn,biorite=info".into(),
        }))
        .insert_resource(SaveFile::default())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(NetworkServerPlugin)
        .run();
}
