use bevy::prelude::*;

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

fn connection_event(
    mut events: EventReader<ConnectionEvent>,
    mut commands: Commands,
    mut state: ResMut<NextState<GameState>>,
    query: Query<Entity, Or<(With<Node>, With<UICamera>)>>,
) {
    let token = handshake(&ARGS).unwrap();
    commands.insert_resource(create_renet_client_from_token(token));
    state.set(GameState::InGame);
}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ConnectionEvent>()
            // .add_system(setup_menu.in_schedule(OnEnter(GameState::Menu)))
            .add_system(connection_event.in_set(OnUpdate(GameState::Menu)));
    }
}
