use super::messages::*;
use bevy_easings::*;
use bevy_rapier3d::na::Vector3;

use crate::*;
use local_ip_address::local_ip;
use smooth_bevy_cameras::LookTransformPlugin;
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
        received_packets_buffer_size: 1000,
        send_channels_config: vec![
            ChannelConfig::Reliable(ReliableChannelConfig {
                packet_budget: 30000,
                max_message_size: 9 * 1024,
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

    RenetClient::new(current_time, socket, connection_config, authentication)
        .unwrap()
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

#[allow(clippy::too_many_arguments)]
pub fn entity_spawn(
    client: ResMut<RenetClient>,
    mut lobby: ResMut<Lobby>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    chunk_messages: ResMut<CurrentClientChunkMessages>,
    _keyboard: Res<Input<KeyCode>>,
    messages: Res<CurrentClientMessages>,
) {
    for message in messages.iter() {
        if let ServerMessage::PlayerSpawn(id) = message {
            info!("Player {} joined the game", id);
            let player_entity = commands.spawn(Player { id: *id }).id();

            if *id == client.client_id() {
                commands
                    .entity(player_entity)
                    .insert(ControlledPlayer)
                    .insert(PbrBundle {
                        transform: Transform::from_xyz(0.0, 50.0, 0.0),
                        ..Default::default()
                    });
            } else {
                commands.entity(player_entity).insert(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Capsule::default())),
                    material: materials.add(Color::rgb(0.8, 0.20, 0.6).into()),
                    transform: Transform::from_xyz(0.0, 50.0, 0.0),
                    ..Default::default()
                });
            }
            insert_player_physics(&mut commands, player_entity);
            lobby.players.insert(*id, player_entity);
        }
    }
    for message in chunk_messages.iter() {
        if let ServerChunkMessage::Init { player_ids } = message {
            debug!("Initializing players");
            for id in player_ids.iter() {
                let player_entity = commands
                    .spawn(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Capsule::default())),
                        material: materials
                            .add(Color::rgb(0.8, 0.20, 0.6).into()),
                        transform: Transform::from_xyz(0.0, 25.0, 0.0),
                        ..Default::default()
                    })
                    .id();

                lobby.players.insert(*id, player_entity);
            }
        }
    }
}

pub fn entity_sync(
    lobby: ResMut<Lobby>,
    mut commands: Commands,
    messages: Res<CurrentClientMessages>,
) {
    for message in messages.iter() {
        #[allow(irrefutable_let_patterns)]
        if let ServerMessage::EntitySync(sync) = message {
            for (player_id, translation) in sync.iter() {
                if let Some(player_entity) = lobby.players.get(player_id) {
                    let transform = Transform {
                        translation: (*translation).into(),
                        ..Default::default()
                    };
                    commands.entity(*player_entity).insert(transform);
                }
            }
        }
    }
}

pub fn update_camera_system(
    players: Query<&Transform, With<ControlledPlayer>>,
    mut cameras: Query<
        (&MainCamera, &mut Transform),
        Without<ControlledPlayer>,
    >,
) {
    if let Ok(player_pos) = &players.get_single() {
        for (_, mut camera_pos) in &mut cameras {
            camera_pos.translation = player_pos.translation;
        }
    }
}

fn movement_axis(
    input: &Res<Input<KeyCode>>,
    plus: KeyCode,
    minus: KeyCode,
) -> f32 {
    let mut axis = 0.0;
    if input.pressed(plus) {
        axis += 1.0;
    }
    if input.pressed(minus) {
        axis -= 1.0;
    }
    axis
}

fn player_input(
    input: Res<Input<KeyCode>>,
    query: Query<(&MainCamera, &Transform), Without<ControlledPlayer>>,
    // mut players: Query<&mut Velocity, With<ControlledPlayer>>,
    mut player_pos: Query<
        (
            &mut ExternalForce,
            &mut ExternalImpulse,
            &Velocity,
            &RapierRigidBodyHandle,
        ),
        With<ControlledPlayer>,
    >,
    mut player_input: ResMut<PlayerInput>,
) {
    for (_options, transform) in query.iter() {
        let (axis_h, axis_v) = (
            movement_axis(&input, KeyCode::W, KeyCode::S),
            movement_axis(&input, KeyCode::A, KeyCode::D),
        );

        let _rotation = transform.rotation;

        let mut f = transform.forward();
        f.y = 0.0;
        let mut l = transform.left();
        l.y = 0.0;
        let vec = ((f * axis_h) + (l * axis_v)).normalize_or_zero();

        player_input.forward = vec.x;
        player_input.sideways = vec.z;
        player_input.jumping = input.pressed(KeyCode::Space);
    }
}

fn client_send_input(
    player_input: Res<PlayerInput>,
    mut is_moving: Local<bool>,
    mut client: ResMut<RenetClient>,
) {
    if player_input.forward != 0.0 && player_input.sideways != 0.0
        || player_input.jumping
    {
        ClientMessage::PlayerInput(*player_input).send(&mut client);
        *is_moving = true;
    } else {
        // no need to send empty inputs multiple times
        if *is_moving {
            ClientMessage::PlayerInput(*player_input).send(&mut client);
            *is_moving = false;
        }
    }
}
pub struct NetworkClientPlugin;

impl Plugin for NetworkClientPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(create_renet_client())
            .add_plugin(EasingsPlugin)
            .init_resource::<CurrentClientMessages>()
            .init_resource::<CurrentClientChunkMessages>()
            .insert_resource(CurrentLocalPlayerChunk {
                chunk_min: IVec3::ZERO,
                world_pos: IVec3::ZERO,
            })
            .insert_resource(PlayerInput::default())
            .insert_resource(AlreadyRequested::default())
            .insert_resource(Lobby::default())
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
                    .with_system(update_camera_system)
                    .with_system(client_recieve_messages)
                    .with_system(entity_spawn)
                    .with_system(player_input)
                    .with_system(receive_chunk)
                    .with_system(request_chunk)
                    .with_system(entity_sync)
                    .with_system(mesher)
                    .with_system(client_ping_test)
                    .with_system(client_send_input)
                    .with_system(update_player_pos)
                    .with_run_criteria(run_if_client_connected),
            )
            .add_plugin(LookTransformPlugin);
    }
}
