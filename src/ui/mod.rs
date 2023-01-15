use crate::prelude::*;
use bevy::asset::LoadState;
use bevy::{prelude::*, winit::WinitSettings};
use lazy_static::lazy_static;

#[derive(Component)]
pub struct MenuElement;

const NORMAL_BUTTON: Color = Color::rgb(1.0, 1.0, 1.0);
const FOCUSED_BUTTON: Color = Color::rgb(0.81, 1.04, 1.40);

#[derive(Component)]
pub enum MenuButton {
    Singleplayer,
    Multiplayer,
    Quit,
}

pub fn test_menu(
    mut app_state: ResMut<State<AppState>>,
    input: Res<Input<KeyCode>>,
) {
    if input.pressed(KeyCode::U) {
        app_state.set(AppState::InGame).unwrap();
    }
}

fn button_hover(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                *color = FOCUSED_BUTTON.into();
            }
            Interaction::Hovered => {
                *color = FOCUSED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

lazy_static! {
    static ref MARGIN: UiRect = UiRect {
        left: Val::Px(0.0),
        top: Val::Px(5.0),
        right: Val::Px(0.0),
        bottom: Val::Px(5.0),
    };
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/Monocraft.otf");
    let texture = asset_server.load("textures/button.png");
    // ui camera
    commands
        .spawn(Camera2dBundle::default())
        .insert(MenuElement);
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .insert(MenuElement)
        .with_children(|parent| {
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(600.0), Val::Px(60.0)),
                        justify_content: JustifyContent::Center,
                        margin: *MARGIN,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    image: UiImage(texture.clone()),
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn(TextBundle::from_section(
                            "Singleplayer",
                            TextStyle {
                                font: font.clone(),
                                font_size: 25.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                            },
                        ))
                        .insert(MenuButton::Singleplayer);
                });
        })
        .with_children(|parent| {
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(600.0), Val::Px(60.0)),
                        justify_content: JustifyContent::Center,
                        margin: *MARGIN,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    image: UiImage(texture.clone()),
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn(TextBundle::from_section(
                            "Multiplayer",
                            TextStyle {
                                font: font.clone(),
                                font_size: 25.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                            },
                        ))
                        .insert(MenuButton::Multiplayer);
                });
        })
        .with_children(|parent| {
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(600.0), Val::Px(60.0)),
                        margin: *MARGIN,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    image: UiImage(texture.clone()),
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn(TextBundle::from_section(
                            "Quit game",
                            TextStyle {
                                font: font.clone(),
                                font_size: 25.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                            },
                        ))
                        .insert(MenuButton::Quit);
                });
        });
}

pub fn menu_unload(
    mut commands: Commands,
    query: Query<Entity, With<MenuElement>>,
) {
    query.iter().for_each(|e| {
        commands.entity(e).despawn_recursive();
    });
}

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(AppState::MainMenu).with_system(setup),
        )
        .add_system_set(
            SystemSet::on_update(AppState::MainMenu)
                .with_system(test_menu)
                .with_system(button_hover),
        )
        .add_system_set(
            SystemSet::on_exit(AppState::MainMenu).with_system(menu_unload),
        );
    }
}
