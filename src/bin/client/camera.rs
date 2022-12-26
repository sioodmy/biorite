use crate::*;
use std::f32::consts;

pub fn spawn_camera(mut commands: Commands) {
    commands
        .spawn((
            Camera3dBundle {
                projection: bevy::render::camera::Projection::Perspective(PerspectiveProjection {
                    fov: consts::PI / 2.,
                    far: 2048.0,
                    ..Default::default()
                }),
                transform: Transform::from_xyz(0.0, 0.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
                ..default()
            },
            AtmosphereCamera::default(),
        ))
        .insert(crate::movement::Player::default());
}
