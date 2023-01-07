use crate::prelude::*;
use bevy::utils::HashMap;
use local_ip_address::local_ip;
use std::{
    net::{SocketAddr, UdpSocket},
    time::{Duration, SystemTime},
};

/// Resource that tracks each player's position
#[derive(Debug, Default, Resource)]
pub struct ServerLobby {
    pub players: HashMap<u64, Entity>,
}

pub fn create_renet_server() -> RenetServer {
    info!("Starting Biorite {} server", env!("CARGO_PKG_VERSION"));
    let server_addr = SocketAddr::new(local_ip().unwrap(), 42069);
    info!("Creating Server! {:?}", server_addr);

    let socket = UdpSocket::bind(server_addr).unwrap();
    // TODO increase block package queue size from default 8
    let connection_config = RenetConnectionConfig {
        max_packet_size: 32 * 1024,
        receive_channels_config: vec![
            ChannelConfig::Unreliable(UnreliableChannelConfig {
                sequenced: true, // We don't care about old positions
                ..Default::default()
            }),
            ChannelConfig::Reliable(ReliableChannelConfig {
                packet_budget: 30000,
                max_message_size: 7000,
                ..Default::default()
            }),
        ],
        ..Default::default()
    };
    let server_config = ServerConfig::new(
        64,
        PROTOCOL_ID,
        server_addr,
        ServerAuthentication::Unsecure,
    );
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    RenetServer::new(current_time, server_config, connection_config, socket)
        .unwrap()
}

fn server_events(
    mut events: EventReader<ServerEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut lobby: ResMut<Lobby>,
    mut server: ResMut<RenetServer>,
) {
    for event in events.iter() {
        match event {
            ServerEvent::ClientConnected(id, _user_data) => {
                info!("Connected {}!", id);
                let player_entity = commands
                    .spawn(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                        material: materials
                            .add(Color::rgb(0.8, 0.7, 0.6).into()),
                        transform: Transform::from_xyz(0.0, 5.0, 0.0),
                        ..Default::default()
                    })
                    .insert(PlayerInput::default())
                    .insert(Player { id: *id })
                    .id();

                // We could send an InitState with all the players id and
                // positions for the client but this is easier
                // to do.
                debug!("sending players {:?}", lobby);
                let mut player_ids = Vec::new();
                for &player_id in lobby.players.keys() {
                    player_ids.push(player_id);
                }
                ServerChunkMessage::Init { player_ids }.send(&mut server, *id);
                lobby.players.insert(*id, player_entity);
                lobby.sent_chunks.insert(*id, Vec::new());
                ServerMessage::PlayerSpawn(*id).broadcast(&mut server);
            }
            ServerEvent::ClientDisconnected(id) => {
                info!("Disconnected {}!", id);
                if let Some(player_entity) = lobby.players.remove(id) {
                    commands.entity(player_entity).despawn();
                }
                ServerMessage::PlayerDespawn(*id).broadcast(&mut server);
            }
        }
    }
}

fn server_sync_players(
    mut server: ResMut<RenetServer>,
    query: Query<(&Transform, &Player)>,
) {
    let mut players: HashMap<u64, [f32; 3]> = HashMap::new();
    for (transform, player) in query.iter() {
        players.insert(player.id, transform.translation.into());
    }

    let sync_message = ServerMessage::EntitySync(players);
    sync_message.broadcast(&mut server);
}

pub fn server_ping_test(
    messages: Res<CurrentServerMessages>,
    mut server: ResMut<RenetServer>,
) {
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

fn move_players_system(
    mut query: Query<(&mut Transform, &PlayerInput)>,
    time: Res<Time>,
) {
    for (mut transform, input) in query.iter_mut() {
        transform.translation.x +=
            input.forward * PLAYER_SPEED * time.delta().as_secs_f32();
        transform.translation.z +=
            input.sideways * PLAYER_SPEED * time.delta().as_secs_f32();
        if input.jumping {
            transform.translation.y += 1.0 * time.delta().as_secs_f32();
        }
    }
}

pub fn server_receive_input(
    messages: Res<CurrentServerMessages>,
    mut server: ResMut<RenetServer>,
    mut commands: Commands,
    mut lobby: ResMut<Lobby>,
) {
    for (id, message) in messages.iter() {
        if let ClientMessage::PlayerInput(input) = message {
            if let Some(player_entity) = lobby.players.get(&id) {
                commands.entity(*player_entity).insert(*input);
            }
        }
    }
}

pub struct NetworkServerPlugin;

impl Plugin for NetworkServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RenetServerPlugin::default())
            .insert_resource(create_renet_client())
            .insert_resource(Lobby::default())
            .init_resource::<CurrentServerMessages>()
            .add_system(crate::server_recieve_messages)
            .add_system(server_ping_test)
            .add_system(server_receive_input)
            .add_system(move_players_system)
            .add_system(server_sync_players)
            .add_system(server_events);
    }
}
