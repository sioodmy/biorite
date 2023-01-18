use crate::prelude::*;
pub use std::collections::HashSet;

pub use bevy::{ecs::prelude::*, utils::HashMap};
pub use bevy_renet::{renet::*, *};
pub use client::*;
pub use serde::{Deserialize, Serialize};
pub use server::*;

pub mod client;
pub mod server;

pub enum Channel {
    Reliable,
    Unreliable,
    Chunk,
}

impl Channel {
    pub fn id(&self) -> u8 {
        match self {
            Channel::Reliable => ReliableChannelConfig::default().channel_id,
            Channel::Unreliable => {
                UnreliableChannelConfig::default().channel_id
            }
            Channel::Chunk => ChunkChannelConfig::default().channel_id,
        }
    }
}

#[derive(Debug, Component)]
pub struct ControlledPlayer;

#[derive(Debug, Component)]
pub struct Player {
    pub id: u64,
}

#[derive(Debug, Default, Resource)]
pub struct Lobby {
    pub players: HashMap<u64, Entity>,
    pub sent_chunks: HashMap<u64, HashSet<IVec3>>,
}

#[derive(
    Debug, Default, Resource, Copy, Clone, Serialize, Deserialize, Component,
)]
pub struct PlayerInput {
    pub forward: f32,
    pub sideways: f32,
    pub jumping: bool,
    pub sneaking: bool,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub motd: String,

    pub player_count: u32,
    pub max_player_count: u32,
}
