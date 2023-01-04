use crate::*;
use bevy::pbr::wireframe::WireframeConfig;
use bevy_egui::{egui, EguiContext};

pub fn wireframe(mut wireframe_config: ResMut<WireframeConfig>, keyboard: Res<Input<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::F3) {
        info!("Toggle debug mode");

        wireframe_config.global = !wireframe_config.global;
    }
}

pub fn display_debug_stats(mut egui: ResMut<EguiContext>, diagnostics: Res<Diagnostics>) {
    egui::Window::new("performance stuff").show(egui.ctx_mut(), |ui| {
        ui.label(format!(
            "Avg. FPS: {:.02}",
            diagnostics
                .get(FrameTimeDiagnosticsPlugin::FPS)
                .unwrap()
                .average()
                .unwrap_or_default()
        ));
        ui.label(format!(
            "Total Entity count: {}",
            diagnostics
                .get(EntityCountDiagnosticsPlugin::ENTITY_COUNT)
                .unwrap()
                .average()
                .unwrap_or_default()
        ));
    });
}
