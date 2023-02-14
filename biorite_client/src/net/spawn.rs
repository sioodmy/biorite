use biorite_shared::{
    net::{data_types::*, protocol::*},
    player::*,
};

use super::util::*;

#[allow(clippy::too_many_arguments)]
pub fn entity_spawn(
    client: ResMut<RenetClient>,
    mut lobby: ResMut<Lobby>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    chunk_messages: ResMut<CurrentClientChunkMessages>,
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
                        transform: Transform::from_xyz(0.0, 60.0, 0.0),
                        ..Default::default()
                    });
            } else {
                commands.entity(player_entity).insert(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Capsule::default())),
                    material: materials.add(Color::rgb(0.8, 0.20, 0.6).into()),
                    transform: Transform::from_xyz(0.0, 70.0, 0.0),
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
