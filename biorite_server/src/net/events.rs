use bevy::prelude::*;
use bevy_renet::renet::*;
use biorite_generator::SaveFile;
use biorite_shared::{
    net::{
        data_types::{Lobby, Player, PlayerInput},
        protocol::*,
    },
    player::insert_player_physics,
};

pub fn server_events(
    mut events: EventReader<ServerEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut lobby: ResMut<Lobby>,
    mut save: ResMut<SaveFile>,
    mut server: ResMut<RenetServer>,
) {
    for event in events.iter() {
        match event {
            ServerEvent::ClientConnected(id, user_data) => {
                let uuid = UserData::from_user_data(user_data);
                let _data = save.get_player_data(uuid.0.clone()).unwrap();
                info!("Connected {}! {:?}", id, uuid.0);
                let player_entity = commands
                    .spawn(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Capsule::default())),
                        material: materials
                            .add(Color::rgb(0.8, 0.20, 0.6).into()),
                        transform: Transform::from_xyz(0.0, 70., 0.0),
                        ..Default::default()
                    })
                    .insert(PlayerInput::default())
                    .insert(Player { id: *id })
                    .id();

                insert_player_physics(&mut commands, player_entity);

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
