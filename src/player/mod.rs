use crate::prelude::*;

pub fn insert_player_physics(commands: &mut Commands, entity: Entity) {
    commands
        .entity(entity)
        .insert(RigidBody::Dynamic)
        .insert(KinematicCharacterController {
            // Don’t allow climbing slopes larger than 45 degrees.
            max_slope_climb_angle: 91.0_f32.to_radians(),
            // Automatically slide down on slopes smaller than 30 degrees.
            min_slope_slide_angle: 91.0_f32.to_radians(),
            snap_to_ground: Some(CharacterLength::Absolute(1.5)),
            ..default()
        })
        .insert(ExternalImpulse::default())
        .insert(ExternalForce::default())
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(Friction {
            coefficient: 0.7,
            combine_rule: CoefficientCombineRule::Min,
        })
        .insert(Restitution {
            coefficient: 1.0,
            combine_rule: CoefficientCombineRule::Min,
        })
        .insert(AdditionalMassProperties::Mass(50.0))
        .insert(Collider::cuboid(0.5, 1.9, 0.5))
        .insert(GravityScale(3.0))
        .insert(Ccd::enabled())
        .insert(Velocity::zero());
}
