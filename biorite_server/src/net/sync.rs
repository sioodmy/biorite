use super::util::*;
use bevy::prelude::*;
use bevy_rapier3d::{na::Vector3, prelude::*};
use biorite_shared::{
    consts::PLAYER_SPEED,
    net::{data_types::*, protocol::*},
};

pub fn move_players_system(
    mut player_pos: Query<
        (
            &mut ExternalForce,
            &mut ExternalImpulse,
            &PlayerInput,
            &Velocity,
            &RapierRigidBodyHandle,
        ),
        &Player,
    >,
    context: Res<RapierContext>,
) {
    for (mut force, mut impulse, input, velocity, handle) in
        player_pos.iter_mut()
    {
        let target_force =
            Vec3::new(input.forward, 0.0, input.sideways) * PLAYER_SPEED;
        force.force = (target_force - velocity.linvel) * 1000.0;
        force.force.y = 0.0;

        if input.jumping {
            // Avoid double jumping by checking gravitational potential energy
            let body = match context.bodies.get(handle.0) {
                Some(b) => b,
                None => continue,
            };
            let e1 = body.gravitational_potential_energy(
                0.001,
                Vector3::new(0.0, -9.81, 0.0),
            );
            let e2 = body.gravitational_potential_energy(
                0.002,
                Vector3::new(0.0, -9.81, 0.0),
            );
            if e1 == e2 {
                impulse.impulse = Vec3::new(0.0, 500.0, 0.0);
            }
        }
    }
}

pub fn server_sync_players(
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

pub fn server_receive_input(
    messages: Res<CurrentServerMessages>,
    mut commands: Commands,
    lobby: ResMut<Lobby>,
) {
    for (id, message) in messages.iter() {
        if let ClientMessage::PlayerInput(input) = message {
            if let Some(player_entity) = lobby.players.get(id) {
                commands.entity(*player_entity).insert(*input);
            }
        }
    }
}
