pub use crate::prelude::*;
pub use bevy::prelude::*;
pub use bevy_rapier3d::prelude::*;
pub use bevy_renet::{renet::*, *};
pub use lz4_flex::{compress_prepend_size, decompress_size_prepended};

mod chunks;
mod debugging;
mod networking;
mod player;
pub mod prelude;
mod render;
mod ui;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    MainMenu,
    InGame,
    Paused,
}

/// For now its hardcoded, might change in future
pub const RENDER_DISTANCE: i16 = 1;

pub const REACH: f32 = 40.0;

pub const REQUEST_LIMIT: usize = 256;

pub const PLAYER_SPEED: f32 = 24.3;

pub const PROTOCOL_ID: u64 = 1000;
