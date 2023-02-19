use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_inspector_egui::egui::Color32;
use egui::{FontFamily, FontId, TextStyle};
use seed15::{phrase::seed_to_seed_phrase, random_seed};

use crate::{
    auth::send_public_key, net::create_renet_client_from_token,
    state::GameState, ARGS,
};

#[derive(Default)]
struct UiState {
    input: String,
    got_seed: bool,
    // seed_input: [String; 15],
    seed_input: String,
}

fn menu(
    mut egui_context: ResMut<EguiContext>,
    mut state: ResMut<State<GameState>>,
    mut ui_state: Local<UiState>,
    mut commands: Commands,
) {
    egui::Window::new("Hello").show(egui_context.ctx_mut(), |ui| {
        ui.label("world");
        ui.horizontal(|ui| {
            let name_label = ui.label("Your name: ");
            ui.text_edit_singleline(&mut ui_state.input)
                .labelled_by(name_label.id);
        });
        if ui_state.got_seed {
            ui.label("Your seedphrase");
            // for (i, word) in ui_state.seed_input.iter_mut().enumerate() {
            //     ui.horizontal(|ui| {
            //         let name_label = ui.label(format!("{}:", i + 1));
            //         ui.text_edit_singleline(&mut *word)
            //             .labelled_by(name_label.id);
            //     });
            // }
            let label = ui.label("seed phrase");
            ui.text_edit_singleline(&mut ui_state.seed_input)
                .labelled_by(label.id);
        }
        // TODO: Implement some actual ui instead of this shit
        if ui.add(egui::Button::new("login")).clicked() {
            ui_state.got_seed = !ui_state.got_seed;
        }
        if ui.add(egui::Button::new("Connect")).clicked() {
            // let phrase = ui_state.seed_input.join(" ");

            let _phrase = ui_state.seed_input.clone();
            // println!("seed {}", phrase);
            let token = send_public_key(&ARGS).unwrap();
            commands.insert_resource(create_renet_client_from_token(token));

            state.overwrite_set(GameState::InGame).unwrap();
        }
    });
}

fn setup_custom_fonts(mut egui_context: ResMut<EguiContext>) {
    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        "my_font".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../../assets/fonts/Monocraft.otf"
        )),
    );

    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "my_font".to_owned());

    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("my_font".to_owned());

    use FontFamily::{Monospace, Proportional};

    let mut style = (*egui_context.ctx_mut().style()).clone();
    style.text_styles = [
        (TextStyle::Heading, FontId::new(25.0, Proportional)),
        (TextStyle::Body, FontId::new(19.0, Proportional)),
        (TextStyle::Monospace, FontId::new(12.0, Monospace)),
        (TextStyle::Button, FontId::new(19.0, Proportional)),
        (TextStyle::Small, FontId::new(8.0, Proportional)),
    ]
    .into();

    egui_context.ctx_mut().set_fonts(fonts);
    egui_context.ctx_mut().set_style(style);
    egui_context.ctx_mut().set_visuals(egui::Visuals {
        window_fill: Color32::from_rgb(187, 127, 86),
        override_text_color: Some(Color32::from_rgb(243, 243, 243)),
        ..Default::default()
    })
}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(EguiPlugin)
            .add_system_set(
                SystemSet::on_enter(GameState::Menu)
                    .with_system(setup_custom_fonts),
            )
            .add_system_set(
                SystemSet::on_update(GameState::Menu).with_system(menu),
            );
    }
}
