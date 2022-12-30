use std::{
    net::{SocketAddr, UdpSocket},
    time::SystemTime,
};

use bevy::{
    log::{Level, LogPlugin},
    math::ivec3,
    pbr::wireframe::WireframePlugin,
};
use bevy_atmosphere::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_spectator::*;
use local_ip_address::local_ip;
use voxelorite::*;

mod camera;
mod chunk;
mod render;

fn create_renet_client() -> RenetClient {
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();

    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();

    let client_id = current_time.as_millis() as u64;

    let connection_config = RenetConnectionConfig::default();

    // TODO Prompt for server IP
    let server_addr = SocketAddr::new(local_ip().unwrap(), 42069);

    let authentication = ClientAuthentication::Unsecure {
        client_id,
        protocol_id: PROTOCOL_ID,
        server_addr,
        user_data: None,
    };

    RenetClient::new(current_time, socket, connection_config, authentication).unwrap()
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        width: 1280.,
                        height: 720.,
                        title: "my cheap shitass minecraft ripoff".to_string(),
                        resizable: true,
                        ..default()
                    },
                    ..default()
                })
                .set(LogPlugin {
                    level: Level::DEBUG,
                    filter: "info,wgpu_core=warn,wgpu_hal=warn".to_string(),
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugin(WireframePlugin)
        .add_plugin(RenetClientPlugin::default())
        .add_plugin(AtmospherePlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(SpectatorPlugin)
        .insert_resource(create_renet_client())
        .insert_resource(Msaa { samples: 4 })
        .add_system(client_ping)
        .add_system(receive_message_system)
        // prototype
        .add_startup_system(camera::spawn_camera)
        .add_startup_system(chunk::spawn_chunk)
        .add_system(chunk::wireframe)
        .insert_resource(ChunkRenderQueue(vec![Chunk::default()]))
        .add_system(render::renderq)
        .run();
}

fn receive_message_system(
    mut client: ResMut<RenetClient>,
    mut render_queue: ResMut<ChunkRenderQueue>,
    mut commands: Commands,
) {
    let reliable_channel_id = ReliableChannelConfig::default().channel_id;

    while let Some(message) = client.receive_message(reliable_channel_id) {
        let server_message = bincode::deserialize(&message).unwrap();

        match server_message {
            ServerMessage::Pong(info) => {
                debug!("Got response from {:?} server", info);
            }
            ServerMessage::Chunk(compressed_chunk) => {
                let chunk = Chunk::from_compressed(compressed_chunk);
                debug!("Got chunk at {:?}", chunk.position);
                render_queue.0.push(chunk);
                debug!("Inserting chunk to render queue");
            }
        }
    }
}

fn request_spawn_chunks(mut client: ResMut<RenetClient>, render_distance: i32) {
    let reliable_channel_id = ReliableChannelConfig::default().channel_id;
    for x in -render_distance..=render_distance {
        for y in -render_distance..=render_distance {
            for z in -render_distance..=render_distance {
                if x * x + y * y + z * z <= render_distance * render_distance {
                    debug!("Requesting chunk at ({}, {}, {})", x, y, z);
                    let ping_message =
                        bincode::serialize(&ClientMessage::RequestChunkData(IVec3 { x, y, z }))
                            .unwrap();

                    client.send_message(reliable_channel_id, ping_message);
                }
            }
        }
    }
}

fn client_ping(mut client: ResMut<RenetClient>, keyboard: Res<Input<KeyCode>>) {
    let reliable_channel_id = ReliableChannelConfig::default().channel_id;

    if keyboard.just_pressed(KeyCode::Space) {
        let ping_message = bincode::serialize(&ClientMessage::Ping).unwrap();

        client.send_message(reliable_channel_id, ping_message);

        debug!("Sent ping!");
    }
    if keyboard.just_pressed(KeyCode::Z) {
        debug!("Requested chunks");
        request_spawn_chunks(client, 2);
    }
}
