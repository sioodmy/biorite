use bevy::{prelude::*, utils::hashbrown::HashMap};
use scc::HashSet;
use serde::{Deserialize, Serialize};

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
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub motd: String,

    pub player_count: u32,
    pub max_player_count: u32,
}
