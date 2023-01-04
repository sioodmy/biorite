use super::messages::*;
use crate::*;
use local_ip_address::local_ip;
use std::{
    net::{SocketAddr, UdpSocket},
    time::SystemTime,
};

pub fn create_renet_client() -> RenetClient {
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();

    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let client_id = current_time.as_millis() as u64;

    let connection_config = RenetConnectionConfig {
        max_packet_size: 32 * 1024,
        send_channels_config: vec![
            ChannelConfig::Reliable(ReliableChannelConfig {
                packet_budget: 30000,
                max_message_size: 8 * 1024,
                ..Default::default()
            }),
            ChannelConfig::Unreliable(UnreliableChannelConfig::default()),
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

    RenetClient::new(current_time, socket, connection_config, authentication).unwrap()
}

pub fn client_ping_test(
    mut client: ResMut<RenetClient>,
    keyboard: Res<Input<KeyCode>>,
    messages: Res<CurrentClientMessages>,
) {
    if keyboard.just_pressed(KeyCode::P) {
        info!("Sending ping!");
        ClientMessage::Ping.send(&mut client);
    }
    for message in messages.iter() {
        #[allow(irrefutable_let_patterns)]
        if let ServerMessage::Pong(info) = message {
            info!("{:?}", info);
        }
    }
}

pub struct NetworkClientPlugin;

impl Plugin for NetworkClientPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(create_renet_client())
            .init_resource::<CurrentClientMessages>()
            .init_resource::<CurrentClientChunkMessages>()
            .insert_resource(CurrentLocalPlayerChunk {
                chunk_min: IVec3::ZERO.into(),
                world_pos: IVec3::ZERO,
            })
            .add_system(update_player_pos)
            .add_system(client_recieve_messages)
            .add_system(chunk_reciever)
            .add_system(new_chunks)
            .add_system(client_ping_test);
    }
}
