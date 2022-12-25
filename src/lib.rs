pub use bevy::prelude::*;
pub use bevy_renet::renet::*;
pub use bevy_renet::*;
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    Ping,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Block {
    Air,
    Stone,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub motd: String,

    pub player_count: u32,
    pub max_player_count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    Pong(ServerInfo),
}

pub const PROTOCOL_ID: u64 = 1000;
