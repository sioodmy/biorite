use crate::prelude::*;
use bevy_egui::{egui, EguiContext};

pub fn display_debug_stats(
    mut egui: ResMut<EguiContext>,
    diagnostics: Res<Diagnostics>,
    player_pos: Res<CurrentLocalPlayerChunk>,
) {
    egui::Window::new("Biorite dev build").show(egui.ctx_mut(), |ui| {
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
        ui.label(format!(
            "Coords: {}/{}/{}",
            player_pos.world_pos.x,
            player_pos.world_pos.y,
            player_pos.world_pos.y
        ));
        ui.label(format!(
            "Chunk: {}/{}/{}",
            player_pos.chunk_min.x / CHUNK_DIM as i32,
            player_pos.chunk_min.y / CHUNK_DIM as i32,
            player_pos.chunk_min.y / CHUNK_DIM as i32
        ));
    });
}