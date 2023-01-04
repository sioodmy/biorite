use crate::*;
use bevy::pbr::wireframe::WireframeConfig;

pub fn wireframe(mut wireframe_config: ResMut<WireframeConfig>, keyboard: Res<Input<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::F3) {
        info!("Toggle debug mode");

        wireframe_config.global = !wireframe_config.global;
    }
}
