use bevy::prelude::*;
use bevy_rapier3d::{na::Vector3, prelude::*};
use biorite_shared::{
    consts::*,
    net::{data_types::*, protocol::*},
};

use crate::camera::MainCamera;

pub fn update_camera_system(
    players: Query<&Transform, With<ControlledPlayer>>,
    mut cameras: Query<
        (&MainCamera, &mut Transform),
        Without<ControlledPlayer>,
    >,
) {
    if let Ok(player_pos) = &players.get_single() {
        for (_, mut camera_pos) in &mut cameras {
            camera_pos.translation =
                // Player collider is smaller than it seems, so move camera
                // up to ~1.8m 
                player_pos.translation + Vec3::new(0., 0.8, 0.);
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

pub fn player_input(
    input: Res<Input<KeyCode>>,
    query: Query<(&MainCamera, &Transform), Without<ControlledPlayer>>,
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
    context: Res<RapierContext>,
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

        // Predict player movement
        for (mut force, mut impulse, velocity, handle) in player_pos.iter_mut()
        {
            let target_force = Vec3::new(vec.x, 0.0, vec.z) * PLAYER_SPEED;
            force.force = (target_force - velocity.linvel) * 1000.0;
            force.force.y = 0.0;

            if input.pressed(KeyCode::Space) {
                // Avoid double jumping by checking gravitational potential
                // energy
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
                    impulse.impulse = Vec3::new(0.0, 900.0, 0.0);
                }
            }
        }
        player_input.forward = vec.x;
        player_input.sideways = vec.z;
        player_input.jumping = input.pressed(KeyCode::Space);
    }
}

pub fn client_send_input(
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
