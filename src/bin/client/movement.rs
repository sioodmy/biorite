use bevy::{input::mouse::MouseMotion, prelude::*};

// Credits to mcpar-land

#[derive(Component)]
pub struct Player {
    pub accel: f32,
    pub max_speed: f32,
    pub sensitivity: f32,
    pub friction: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub fov: f32,
    pub velocity: Vec3,
    pub key_forward: KeyCode,
    pub key_backward: KeyCode,
    pub key_left: KeyCode,
    pub key_right: KeyCode,
    pub key_up: KeyCode,
    pub key_down: KeyCode,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            // TODO de-hardcode these values
            accel: 1.2,
            max_speed: 5.3,
            sensitivity: 8.0,
            friction: 1.0,
            pitch: 0.0,
            yaw: 0.3,
            fov: 88.0,
            velocity: Vec3::ZERO,
            key_forward: KeyCode::W,
            key_backward: KeyCode::S,
            key_left: KeyCode::A,
            key_right: KeyCode::D,
            key_up: KeyCode::Space,
            key_down: KeyCode::LShift,
        }
    }
}

fn forward_vector(rotation: &Quat) -> Vec3 {
    rotation.mul_vec3(Vec3::Z).normalize()
}

fn forward_walk_vector(rotation: &Quat) -> Vec3 {
    let f = forward_vector(rotation);
    Vec3::new(f.x, 0.0, f.z).normalize()
}

fn strafe_vector(rotation: &Quat) -> Vec3 {
    // Rotate it 90 degrees to get the strafe direction
    Quat::from_rotation_y(90.0f32.to_radians())
        .mul_vec3(forward_walk_vector(rotation))
        .normalize()
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

fn camera_movement_system(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Player, &mut Transform)>,
) {
    for (mut options, mut transform) in query.iter_mut() {
        let (axis_h, axis_v, axis_float) = (
            movement_axis(&keyboard_input, options.key_right, options.key_left),
            movement_axis(
                &keyboard_input,
                options.key_backward,
                options.key_forward,
            ),
            movement_axis(&keyboard_input, options.key_up, options.key_down),
        );

        let rotation = transform.rotation;
        let accel: Vec3 = (strafe_vector(&rotation) * axis_h)
            + (forward_walk_vector(&rotation) * axis_v)
            + (Vec3::Y * axis_float);
        let accel: Vec3 = if accel.length() != 0.0 {
            accel.normalize() * options.accel
        } else {
            Vec3::ZERO
        };

        let friction: Vec3 = if options.velocity.length() != 0.0 {
            options.velocity.normalize() * -1.0 * options.friction
        } else {
            Vec3::ZERO
        };

        options.velocity += accel * time.delta_seconds();

        // clamp within max speed
        if options.velocity.length() > options.max_speed {
            options.velocity = options.velocity.normalize() * options.max_speed;
        }

        let delta_friction = friction * time.delta_seconds();

        options.velocity = if (options.velocity + delta_friction).signum()
            != options.velocity.signum()
        {
            Vec3::ZERO
        } else {
            options.velocity + delta_friction
        };

        transform.translation += options.velocity;
    }
}

pub fn mouse_movement(
    time: Res<Time>,
    mut mouse_motion_event: EventReader<MouseMotion>,
    mut query: Query<(&mut Player, &mut Transform)>,
) {
    let mut delta: Vec2 = Vec2::ZERO;
    for event in mouse_motion_event.iter() {
        delta += event.delta;
    }
    if delta.is_nan() {
        return;
    }

    for (mut options, mut transform) in query.iter_mut() {
        options.yaw -= delta.x * options.sensitivity * time.delta_seconds();
        options.pitch += delta.y * options.sensitivity * time.delta_seconds();

        options.pitch = options.pitch.clamp(-options.fov, options.fov);
        // println!("pitch: {}, yaw: {}", options.pitch, options.yaw);

        let yaw_radians = options.yaw.to_radians();
        let pitch_radians = options.pitch.to_radians();

        transform.rotation = Quat::from_axis_angle(Vec3::Y, yaw_radians)
            * Quat::from_axis_angle(-Vec3::X, pitch_radians);
    }
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(mouse_movement)
            .add_system(camera_movement_system);
    }
}
