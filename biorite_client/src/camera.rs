use crate::raycast::ChunkRaycast;
use bevy::input::mouse::MouseMotion;
use bevy_atmosphere::prelude::AtmosphereCamera;
use bevy_mod_raycast::RaycastSource;
use biorite_shared::consts::RENDER_DISTANCE;
use std::f32::consts;

use crate::*;

#[derive(Component)]
pub struct Camera {
    yaw: f32,
    pitch: f32,
    fov: f32,
    sensitivity: f32,
}
impl Default for Camera {
    fn default() -> Self {
        Camera {
            yaw: 0.3,
            pitch: 0.3,
            fov: 90.0,
            sensitivity: 8.0,
        }
    }
}

pub fn spawn_window(mut commands: Commands) {
    commands.spawn(Window {
        resolution: (1920., 1080.).into(),
        transparent: false,
        title: format!("Biorite {}", env!("CARGO_PKG_VERSION")),
        resizable: true,
        present_mode: PresentMode::AutoVsync,
        ..Default::default()
    });
}

#[derive(Component)]
pub struct MainCamera;

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            projection: bevy::render::camera::Projection::Perspective(
                PerspectiveProjection {
                    fov: consts::PI / 2.3,
                    // Render distance - 1
                    far: RENDER_DISTANCE as f32 * 2.0 * 64.0
                        - RENDER_DISTANCE as f32 / 2.0,
                    ..Default::default()
                },
            ),
            transform: Transform::from_xyz(-1.0, 1.0, -1.0)
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        AtmosphereCamera::default(),
        FogSettings {
            color: Color::rgba(0.1, 0.2, 0.4, 1.0),
            falloff: FogFalloff::Linear {
                start: 0.8,
                end: 2.2,
            },
            ..Default::default()
        },
        RaycastSource::<ChunkRaycast>::new_transform_empty(),
        MainCamera,
        Camera::default(),
    ));
}

pub fn crosshair(mut commands: Commands, assets: Res<AssetServer>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.), Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(ImageBundle {
                image: assets.load("textures/crosshair.png").into(),
                ..default()
            });
        });
}

pub fn mouse_movement(
    time: Res<Time>,
    mut mouse_motion_event: EventReader<MouseMotion>,
    mut query: Query<(&mut Camera, &mut Transform)>,
) {
    let mut delta: Vec2 = Vec2::ZERO;
    for event in mouse_motion_event.iter() {
        delta += event.delta;
    }
    if delta.is_nan() {
        return;
    }

    for (mut cam, mut transform) in query.iter_mut() {
        cam.yaw -= delta.x * cam.sensitivity * time.delta_seconds();
        cam.pitch += delta.y * cam.sensitivity * time.delta_seconds();

        cam.pitch = cam.pitch.clamp(-cam.fov, cam.fov);
        // println!("pitch: {}, yaw: {}", pitch, yaw);

        let yaw_radians = cam.yaw.to_radians();
        let pitch_radians = cam.pitch.to_radians();

        transform.rotation = Quat::from_axis_angle(Vec3::Y, yaw_radians)
            * Quat::from_axis_angle(-Vec3::X, pitch_radians);
    }
}
