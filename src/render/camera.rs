use bevy_spectator::Spectator;
use std::f32::consts;

use crate::*;

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            projection: bevy::render::camera::Projection::Perspective(PerspectiveProjection {
                fov: consts::PI / 2.,
                far: 2048.0,
                ..Default::default()
            }),
            transform: Transform::from_xyz(1.0, 0.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        AtmosphereCamera::default(),
        Spectator,
    ));
}
