use bevy_spectator::Spectator;
use std::f32::consts;

use crate::*;

#[derive(Component)]
pub struct Camera;

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            projection: bevy::render::camera::Projection::Perspective(
                PerspectiveProjection {
                    fov: consts::PI / 2.3,
                    // Render distance - 1
                    far: RENDER_DISTANCE as f32 * 2.0 * 16.0
                        - RENDER_DISTANCE as f32 / 2.0,
                    ..Default::default()
                },
            ),
            transform: Transform::from_xyz(-1.0, 1.0, -1.0)
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        AtmosphereCamera::default(),
        Camera,
    ));
}
