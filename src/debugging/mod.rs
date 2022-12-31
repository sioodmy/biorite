pub use bevy::{pbr::wireframe::WireframePlugin, prelude::*};
pub use bevy_inspector_egui::WorldInspectorPlugin;

pub mod wireframe;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(wireframe::wireframe)
            .add_plugin(WireframePlugin)
            .add_plugin(WorldInspectorPlugin::new());
    }
}
