use belly::{prelude::*, core::Widgets};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_inspector_egui::egui::Color32;
use egui::{FontFamily, FontId, TextStyle};

use crate::{
    auth::handshake, net::create_renet_client_from_token, state::GameState,
    ARGS,
};

#[derive(Default)]
struct UiState {
    input: String,
    got_seed: bool,
    // seed_input: [String; 15],
    seed_input: String,
}

pub struct ConnectionEvent {
    pub ip: String,
}

#[derive(Component)]
pub struct UICamera;

#[derive(Component, Default)]
pub struct SeedPhrase(pub [String; 15]);

fn setup_menu(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default()).insert(UICamera);
    let that = commands.spawn_empty().id();
    let input = commands.spawn_empty().id();
    let button = commands.spawn_empty().id();
    commands.add(eml! {
         <body s:padding="20px">
        <div c:top>
        <span id="title">"Biorite"</span>
        </div>
        <br/>
        <buttongroup on:value_change=connect!(|ctx| {
            let ev = ctx.event();
            ctx.select(ev.old_value()).add_class("hidden");
            ctx.select(ev.new_value()).remove_class("hidden");
        })>
          <button value=".tab1" pressed>"Multiplayer"</button>
          <button value=".tab2">"Tab 2"</button>
          <button value=".tab3">"Tab 3"</button>
        </buttongroup>
        <br/>
        <div c:content>
          <div c:tab1>
        <span>"Server address: "</span>
        <br />
            <textinput {input} s:width="150px"/>
        <br/>
         <button {button} on:press=connect!(|ctx| {
                    ctx.send_event(ConnectionEvent { ip: "127.0.0.1:42069".to_string() })
                })>
                    "Connect"
                </button>
            <brl/>

        </div>
          <div c:tab2 c:hidden>
        <span>"Seedphrase"</span>
        <br/>
        <for i in=1..=15>
            <span>{i.to_string()}":"</span>
            <textinput s:width="150px"/>
            <br/>
            </for>
        
          </div>
          <div c:tab3 c:hidden>"Tab 3 content"</div>
        </div>
      </body>
    });
    commands.add(StyleSheet::parse(
        r#"
        * {
            font: "fonts/Monocraft.otf";
        }
        body {
            background-color: #a6d189;
        }
        #title {
            font-size: 64px;
            color: #232634;
        }
        .top {
            justify-content: center;
            padding: 50px;
        }
        .sidebar {
            max-width: 38%;
            background-color: #303446;
            padding: 20px;
            padding-bottom: 100%;
        }
        div: {
            justify-content: center;
        }
        .counter {
            max-width: 200px;
            justify-content: space-between;
        }
        .hidden {
            display: none;
        }
        .button-foreground {
            background-color: #5b6078;
            color: #cad3f5;
            padding: 10px;
        }
        button:hover .button-foreground {
            background-color: #494d64;
        }
        button:pressed .button-foreground {
            background-color: #363a4f;
        }
        .text-input {
            justify-content: flex-start;
            padding: 5px;
            margin: 10px;
        }
        .text-input-border{
            background-color: #5b6078;
            color: #cad3f5;           
            padding: 1px;
        }
       .text-input-cursor {
            top: 1px;
            bottom: 1px;
            background-color: #2f2f2f;
        }
        .text-input-background {
            background-color: #5b6078;
            color: #cad3f5;           
        }
       
    "#,
    ));
}

fn connection_event(mut events: EventReader<ConnectionEvent>, mut commands: Commands, mut state: ResMut<State<GameState>>, query: Query<Entity, Or<(With<Node>, With<UICamera>)>>){
    for event in events.iter() {
        info!("connection event {:?}", event.ip);
        let token = handshake(&ARGS).unwrap();
        commands.insert_resource(create_renet_client_from_token(token));
    for entity in query.iter(){
        commands.entity(entity).despawn();
    }

        state.overwrite_set(GameState::InGame).unwrap();
    }
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
            .add_event::<ConnectionEvent>()
            .add_plugin(BellyPlugin)
            .add_system_set(
                SystemSet::on_enter(GameState::Menu)
                    .with_system(setup_custom_fonts)
                    .with_system(setup_menu),
            )
            .add_system_set(
                SystemSet::on_update(GameState::Menu)
                    .with_system(connection_event),
            );
    }
}
