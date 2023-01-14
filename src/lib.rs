pub use crate::prelude::*;
pub use bevy::prelude::*;
pub use bevy_rapier3d::prelude::*;
pub use bevy_renet::{renet::*, *};
use lz4::block::{compress, CompressionMode};

mod chunks;
mod debugging;
mod networking;
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
pub const RENDER_DISTANCE: i16 = 3;

pub const PLAYER_SPEED: f32 = 24.3;

pub const PROTOCOL_ID: u64 = 1000;
