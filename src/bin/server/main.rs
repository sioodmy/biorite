use bevy::log::LogPlugin;
use local_ip_address::local_ip;
use std::{
    net::{SocketAddr, UdpSocket},
    time::SystemTime,
};
use voxelorite::*;

fn create_renet_server() -> RenetServer {
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();

    /* Public hosting, requires port forwarding
    let rt = tokio::runtime::Runtime::new().unwrap();
    let public_ip = rt.block_on(public_ip::addr()).unwrap();
    let server_addr = SocketAddr::new(public_ip, 42069);
    */

    let server_addr = SocketAddr::new(local_ip().unwrap(), 42069);
    info!("Creating Server! {:?}", server_addr);

    let server_config =
        ServerConfig::new(64, PROTOCOL_ID, server_addr, ServerAuthentication::Unsecure);

    let connection_config = RenetConnectionConfig::default();

    let inbound_server_addr = SocketAddr::new(local_ip().unwrap(), 42069);
    let socket = UdpSocket::bind(inbound_server_addr).unwrap();

    RenetServer::new(current_time, server_config, connection_config, socket).unwrap()
}

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugin(LogPlugin {
            level: bevy::log::Level::INFO,
            filter: "info,wgpu_core=warn,wgpu_hal=warn".into(),
        })
        .add_plugin(RenetServerPlugin::default())
        .insert_resource(create_renet_server())
        .add_system(server_events)
        .add_system(receive_message_system)
        .run();
}

fn server_events(mut events: EventReader<ServerEvent>) {
    for event in events.iter() {
        match event {
            ServerEvent::ClientConnected(id, _user_data) => info!("Connected {}!", id),
            ServerEvent::ClientDisconnected(id) => info!("Disconnected {}!", id),
        }
    }
}

fn receive_message_system(mut server: ResMut<RenetServer>) {
    let reliable_channel_id = ReliableChannelConfig::default().channel_id;

    for client_id in server.clients_id().into_iter() {
        while let Some(message) = server.receive_message(client_id, reliable_channel_id) {
            let client_message = bincode::deserialize(&message).unwrap();
            match client_message {
                ClientMessage::Ping => {
                    info!("Got ping from {}!", client_id);
                    let pong = bincode::serialize(&ServerMessage::Pong(ServerInfo {
                        name: "Test server".to_string(),
                        motd: "Just testing".to_string(),
                        player_count: 1,
                        max_player_count: 10,
                    }))
                    .unwrap();
                    server.send_message(client_id, reliable_channel_id, pong);
                }
            }
        }
    }
}
