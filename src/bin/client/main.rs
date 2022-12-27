use bevy::{
    log::{Level, LogPlugin},
    pbr::wireframe::WireframePlugin,
};

use bevy_atmosphere::prelude::*;
use bevy_spectator::*;
use local_ip_address::local_ip;
use std::{
    net::{SocketAddr, UdpSocket},
    time::SystemTime,
};
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

    //TODO Prompt for server IP
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
        .add_plugin(SpectatorPlugin)
        .insert_resource(create_renet_client())
        .insert_resource(Msaa { samples: 4 })
        .add_system(client_ping)
        .add_system(receive_message_system)
        // prototype
        .add_startup_system(camera::spawn_camera)
        .add_startup_system(chunk::spawn_chunk)
        .add_system(chunk::wireframe)
        .run();
}

fn receive_message_system(mut client: ResMut<RenetClient>) {
    let reliable_channel_id = ReliableChannelConfig::default().channel_id;
    while let Some(message) = client.receive_message(reliable_channel_id) {
        let server_message = bincode::deserialize(&message).unwrap();
        match server_message {
            ServerMessage::Pong(info) => {
                info!("Got response from {:?} server", info);
            }
        }
    }
}
fn client_ping(mut client: ResMut<RenetClient>, keyboard: Res<Input<KeyCode>>) {
    let reliable_channel_id = ReliableChannelConfig::default().channel_id;
    if keyboard.just_pressed(KeyCode::Space) {
        let ping_message = bincode::serialize(&ClientMessage::Ping).unwrap();
        client.send_message(reliable_channel_id, ping_message);
        info!("Sent ping!");
    }
}
