use bevy::log::LogPlugin;
use biorite::*;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugin(LogPlugin {
            level: bevy::log::Level::ERROR,
            filter: "error,wgpu_core=warn,wgpu_hal=warn,biorite=info".into(),
        })
        .insert_resource(create_renet_server())
        .add_plugin(NetworkServerPlugin)
        .add_plugin(ChunkServerPlugin)
        // .insert_resource(create_renet_server())
        .run();
}
