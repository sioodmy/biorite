pub use bevy::{
    diagnostic::{Diagnostics, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin},
    pbr::wireframe::WireframePlugin,
    prelude::*,
};
pub use bevy_egui::EguiPlugin;
pub use bevy_inspector_egui::WorldInspectorPlugin;
pub use wireframe::*;

pub mod wireframe;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(wireframe::wireframe)
            .add_plugin(WireframePlugin)
            .add_plugin(FrameTimeDiagnosticsPlugin)
            .add_plugin(EntityCountDiagnosticsPlugin)
            .add_plugin(EguiPlugin)
            .add_plugin(WorldInspectorPlugin::new())
            .add_system(display_debug_stats);
    }
}
