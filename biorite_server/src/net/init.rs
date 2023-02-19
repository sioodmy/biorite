use super::*;
use crate::{
    block_update::handle_block_updates, chunks::ChunkServerPlugin, ARGS,
    PRIVATE_KEY,
};
use bevy::prelude::*;
use bevy_renet::renet::*;
use biorite_shared::{
    net::{data_types::*, protocol::*},
};

use std::{
    net::{UdpSocket},
    time::{Duration, SystemTime},
};

pub fn create_renet_server() -> RenetServer {
    info!("Starting Biorite {} server", env!("CARGO_PKG_VERSION"));
    let server_addr = parse_ip(&ARGS.ip);
    info!("Creating Server! {:?}", server_addr);

    let socket = UdpSocket::bind(server_addr).unwrap();
    let connection_config = RenetConnectionConfig {
        max_packet_size: 33 * 1024,
        received_packets_buffer_size: 9000,
        sent_packets_buffer_size: 10000,
        receive_channels_config: vec![
            ChannelConfig::Unreliable(UnreliableChannelConfig {
                sequenced: true, // We don't care about old positions
                ..Default::default()
            }),
            ChannelConfig::Reliable(ReliableChannelConfig {
                packet_budget: 10000,
                max_message_size: 5000,
                message_send_queue_size: 1024 * 5,
                ..Default::default()
            }),
            ChannelConfig::Chunk(ChunkChannelConfig {
                resend_time: Duration::from_millis(100),
                max_message_size: 1024 * 1024,
                message_send_queue_size: 10000,
                ..Default::default()
            }),
        ],
        ..Default::default()
    };
    let server_config = ServerConfig::new(
        64,
        PROTOCOL_ID,
        server_addr,
        ServerAuthentication::Secure {
            private_key: *PRIVATE_KEY,
        },
    );
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    RenetServer::new(current_time, server_config, connection_config, socket)
        .unwrap()
}

pub struct NetworkServerPlugin;

impl Plugin for NetworkServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RenetServerPlugin::default())
            .insert_resource(create_renet_server())
            .insert_resource(Lobby::default())
            .add_plugin(ChunkServerPlugin)
            .init_resource::<CurrentServerMessages>()
            .add_system(server_recieve_messages)
            .add_system(handle_block_updates)
            .add_system(server_receive_input)
            .add_system(move_players_system)
            .add_system(server_events)
            .add_system(server_sync_players);
    }
}
