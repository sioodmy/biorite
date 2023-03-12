use crate::state::GameState;

use super::*;

use bevy_easings::EasingsPlugin;
use biorite_shared::net::{data_types::*, protocol::*};
use local_ip_address::local_ip;
use std::{
    net::{SocketAddr, UdpSocket},
    time::SystemTime,
};

pub fn create_renet_client_from_token(
    connect_token: ConnectToken,
) -> RenetClient {
    let client_addr = SocketAddr::from(([127, 0, 0, 1], 0));
    let socket = UdpSocket::bind(client_addr).unwrap();
    let connection_config = RenetConnectionConfig::default();

    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let authentication = ClientAuthentication::Secure { connect_token };

    RenetClient::new(current_time, socket, connection_config, authentication)
        .unwrap()
}

pub fn create_renet_client() -> RenetClient {
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();

    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let client_id = current_time.as_millis() as u64;

    let connection_config = RenetConnectionConfig {
        max_packet_size: 48 * 1024,
        received_packets_buffer_size: 9000,
        sent_packets_buffer_size: 1000,
        send_channels_config: vec![
            ChannelConfig::Reliable(ReliableChannelConfig {
                packet_budget: 40000,
                max_message_size: 32 * 1024,
                message_receive_queue_size: 1024 * 15,
                ..Default::default()
            }),
            ChannelConfig::Unreliable(UnreliableChannelConfig {
                sequenced: true,
                ..Default::default()
            }),
        ],
        ..Default::default()
    };
    // TODO Prompt for server IP
    let server_addr = SocketAddr::new(local_ip().unwrap(), 42069);

    let authentication = ClientAuthentication::Unsecure {
        client_id,
        protocol_id: PROTOCOL_ID,
        server_addr,
        user_data: None,
    };

    RenetClient::new(current_time, socket, connection_config, authentication)
        .unwrap()
}

pub struct NetworkClientPlugin;

impl Plugin for NetworkClientPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(create_renet_client())
            .add_plugin(EasingsPlugin)
            .init_resource::<CurrentClientMessages>()
            .init_resource::<CurrentClientChunkMessages>()
            .insert_resource(PlayerInput::default())
            .insert_resource(AlreadyRequested::default())
            .insert_resource(Lobby::default())
            .add_systems(
                (
                    update_camera_system,
                    client_recieve_messages,
                    entity_spawn,
                    disconnect_on_exit,
                    player_input,
                    receive_chunk,
                    request_chunk,
                    entity_sync,
                    client_send_input,
                )
                    .in_set(OnUpdate(GameState::InGame)),
            );
    }
}
