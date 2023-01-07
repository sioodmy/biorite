use super::messages::*;
use crate::*;
use local_ip_address::local_ip;
use smooth_bevy_cameras::{
    LookTransform, LookTransformBundle, LookTransformPlugin, Smoother,
};
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
pub fn entity_spawn(
    mut client: ResMut<RenetClient>,
    mut lobby: ResMut<Lobby>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    chunk_messages: ResMut<CurrentClientChunkMessages>,
    keyboard: Res<Input<KeyCode>>,
    messages: Res<CurrentClientMessages>,
) {
    for message in messages.iter() {
        if let ServerMessage::PlayerSpawn(id) = message {
            info!("Player {} joined the game", id);
            let player_entity = commands
                .spawn(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                    material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                    transform: Transform::from_xyz(3.0, 5.0, 3.0),
                    ..Default::default()
                })
                .id();

            if *id == client.client_id() {
                commands.entity(player_entity).insert(ControlledPlayer);
            }

            lobby.players.insert(*id, player_entity);
        }
    }
    for message in chunk_messages.iter() {
        if let ServerChunkMessage::Init { player_ids } = message {
            debug!("Initializing players");
            for id in player_ids.iter() {
                let player_entity = commands
                    .spawn(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                        material: materials
                            .add(Color::rgb(0.8, 0.7, 0.6).into()),
                        transform: Transform::from_xyz(3.0, 0.5, 3.0),
                        ..Default::default()
                    })
                    .id();

                lobby.players.insert(*id, player_entity);
            }
        }
    }
}

pub fn entity_sync(
    mut client: ResMut<RenetClient>,
    mut lobby: ResMut<Lobby>,
    mut commands: Commands,
    keyboard: Res<Input<KeyCode>>,
    mut query: Query<(&Camera, &Transform)>,
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
    players: Query<(&ControlledPlayer, &Transform)>,
    mut cameras: Query<(&Camera, &mut Transform), Without<ControlledPlayer>>,
) {
    for (_, player_pos) in &players {
        for (_, mut camera_pos) in &mut cameras {
            *camera_pos = Transform::from_translation(
                player_pos.translation + Vec3::new(10.0, 30.0, 20.0),
            )
            .looking_at(player_pos.translation, Vec3::Y);
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
    mut query: Query<(&Camera, &Transform)>,
    mut player_input: ResMut<PlayerInput>,
) {
    for (mut options, mut transform) in query.iter() {
        let (axis_h, axis_v) = (
            movement_axis(&input, KeyCode::W, KeyCode::S),
            movement_axis(&input, KeyCode::A, KeyCode::D),
        );

        let rotation = transform.rotation;

        let mut f = transform.forward();
        f.y = 0.0;
        let mut l = transform.left();
        l.y = 0.0;
        let vec = ((f * axis_h) + (l * axis_v)).normalize_or_zero();
        player_input.forward = vec.x;
        player_input.sideways = vec.z;
        player_input.jumping = input.just_pressed(KeyCode::Space);
        player_input.sneaking = input.pressed(KeyCode::LShift);
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
        debug!("sending movement moving: {:?}", is_moving);
    } else {
        // no need to send empty inputs multiple times
        if *is_moving {
            ClientMessage::PlayerInput(*player_input).send(&mut client);
            *is_moving = false;
            debug!("sending stop signal");
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
            .insert_resource(PlayerInput::default())
            .insert_resource(Lobby::default())
            .add_system(update_player_pos)
            .add_plugin(LookTransformPlugin)
            .add_system(client_send_input)
            .add_system(update_camera_system)
            .add_system(client_recieve_messages)
            .add_system(entity_spawn)
            // .add_system(chunk_reciever)
            .add_system(player_input)
            .add_system(entity_sync)
            // .add_system(new_chunks)
            .add_system(client_ping_test);
    }
}
