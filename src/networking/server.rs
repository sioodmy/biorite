use crate::prelude::*;
use local_ip_address::local_ip;
use std::{
    net::{SocketAddr, UdpSocket},
    time::SystemTime,
};

pub fn create_renet_server() -> RenetServer {
    info!("Starting Biorite {} server", env!("CARGO_PKG_VERSION"));
    let server_addr = SocketAddr::new(local_ip().unwrap(), 42069);
    info!("Creating Server! {:?}", server_addr);

    let socket = UdpSocket::bind(server_addr).unwrap();
    // TODO increase block package queue size from default 8
    let connection_config = RenetConnectionConfig {
        max_packet_size: 32 * 1024,
        receive_channels_config: vec![
            ChannelConfig::Unreliable(UnreliableChannelConfig::default()),
            ChannelConfig::Reliable(ReliableChannelConfig {
                packet_budget: 30000,
                max_message_size: 7000,
                ..Default::default()
            }),
        ],
        ..Default::default()
    };
    let server_config =
        ServerConfig::new(64, PROTOCOL_ID, server_addr, ServerAuthentication::Unsecure);
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    RenetServer::new(current_time, server_config, connection_config, socket).unwrap()
}

pub fn server_connection(mut server_events: EventReader<ServerEvent>) {
    for event in server_events.iter() {
        match event {
            ServerEvent::ClientConnected(id, _) => {
                // visualizer.add_client(*id);
                info!("Connected {}!", id)
            }
            ServerEvent::ClientDisconnected(id) => {
                info!("Disconnected {}!", id)
            }
        }
    }
}

fn server_events(mut events: EventReader<ServerEvent>) {
    for event in events.iter() {
        match event {
            ServerEvent::ClientConnected(id, _user_data) => {
                info!("Connected {}!", id)
            }
            ServerEvent::ClientDisconnected(id) => {
                info!("Disconnected {}!", id)
            }
        }
    }
}

pub fn server_ping_test(messages: Res<CurrentServerMessages>, mut server: ResMut<RenetServer>) {
    for (id, message) in messages.iter() {
        if matches!(message, ClientMessage::Ping) {
            info!("Got ping from {}!", id);
            ServerMessage::Pong(ServerInfo {
                name: "Test".to_string(),
                motd: "Test server".to_string(),
                player_count: 1,
                max_player_count: 5,
            })
            .send(&mut server, *id);
        }
    }
}

pub struct NetworkServerPlugin;

impl Plugin for NetworkServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RenetServerPlugin::default())
            .insert_resource(create_renet_client())
            .insert_resource(get_noise())
            .init_resource::<CurrentServerMessages>()
            .add_system(crate::server_recieve_messages)
            .add_system(server_ping_test)
            .add_system(server_events);
    }
}
