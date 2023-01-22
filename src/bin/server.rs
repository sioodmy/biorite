use bevy::{log::LogPlugin, render::settings::WgpuSettings};
use bevy_rapier3d::prelude::*;
use biorite::*;

fn main() {
    App::new()
        // .insert_resource(WgpuSettings {
        //     backends: None,
        //     ..Default::default()
        // })
        .add_plugins(DefaultPlugins.set(LogPlugin {
            level: bevy::log::Level::ERROR,
            filter: "error,wgpu_core=warn,wgpu_hal=warn,biorite=info".into(),
        }))
        .insert_resource(create_renet_server())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(NetworkServerPlugin)
        .add_plugin(ChunkServerPlugin)
        .run();
}
