use crate::prelude::*;
use bevy::{ecs::prelude::*, utils::HashMap};
pub use bevy_renet::{renet::*, *};
pub use serde::{Deserialize, Serialize};

#[derive(Debug, Component)]
pub struct Player {
    pub id: u64,
}

#[derive(Debug, Default, Resource)]
pub struct Lobby {
    pub players: HashMap<u64, Entity>,
}

// https://wiki.vg/Protocol#Player_Input
#[derive(
    Debug, Default, Resource, Copy, Clone, Serialize, Deserialize, Component,
)]
pub struct PlayerInput {
    pub forward: f32,
    pub sideways: f32,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub motd: String,

    pub player_count: u32,
    pub max_player_count: u32,
}

#[derive(Debug, Default, Deref, DerefMut, Resource)]
pub struct CurrentServerMessages(Vec<(u64, ClientMessage)>);

#[derive(Debug, Default, Deref, DerefMut, Resource)]
pub struct CurrentClientMessages(Vec<ServerMessage>);

#[derive(Default, Deref, DerefMut, Resource)]
pub struct CurrentClientChunkMessages(Vec<ServerChunkMessage>);

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

#[derive(Debug, Serialize, Deserialize, Resource)]
pub enum ServerChunkMessage {
    /// Single chunk
    Chunk(CompressedChunk),
    /// Multiple chunks when, for example spawn chunks
    ChunkBatch(Vec<CompressedChunk>),
    // TODO: send spawn chunks and some stuff
    Init {
        player_ids: Vec<u64>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    Ping,
    RequestChunk(IVec3),
    RequestChunkBatch(Vec<IVec3>),
    PlayerInput(PlayerInput),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    Pong(ServerInfo),
    PlayerSpawn(u64),
    PlayerDespawn(u64),
    EntitySync(HashMap<u64, [f32; 3]>),
}

impl ServerChunkMessage {
    pub fn send(&self, server: &mut RenetServer, id: u64) {
        let message = bincode::serialize(self).unwrap();
        debug!("Sending message");
        server.send_message(id, Channel::Chunk.id(), message);
    }
}

impl ServerMessage {
    pub fn send(&self, server: &mut RenetServer, id: u64) {
        let message = bincode::serialize(self).unwrap();
        if matches!(self, ServerMessage::EntitySync(_)) {
            server.send_message(id, Channel::Unreliable.id(), message);
        } else {
            server.send_message(id, Channel::Reliable.id(), message);
        }
    }
    pub fn broadcast(&self, server: &mut RenetServer) {
        let message = bincode::serialize(self).unwrap();
        if matches!(self, ServerMessage::EntitySync(_)) {
            server.broadcast_message(Channel::Unreliable.id(), message);
        } else {
            server.broadcast_message(Channel::Reliable.id(), message);
        }
    }
    pub fn broadcast_except(&self, server: &mut RenetServer, id: u64) {
        let message = bincode::serialize(self).unwrap();
        server.broadcast_message_except(id, Channel::Reliable.id(), message);
    }
}

pub fn server_recieve_messages(
    mut server: ResMut<RenetServer>,
    mut messages: ResMut<CurrentServerMessages>,
) {
    messages.0.clear();
    for channel in [Channel::Reliable, Channel::Unreliable] {
        for client_id in server.clients_id().into_iter() {
            while let Some(message) =
                server.receive_message(client_id, channel.id())
            {
                let client_message = bincode::deserialize(&message).unwrap();
                messages.0.push((client_id, client_message));
            }
        }
    }
}

pub fn client_recieve_messages(
    mut client: ResMut<RenetClient>,
    mut messages: ResMut<CurrentClientMessages>,
    mut chunk_messages: ResMut<CurrentClientChunkMessages>,
) {
    messages.0.clear();
    chunk_messages.0.clear();
    for channel in [Channel::Reliable, Channel::Unreliable] {
        while let Some(message) = client.receive_message(channel.id()) {
            let server_message = bincode::deserialize(&message).unwrap();
            messages.0.push(server_message);
        }
    }
    while let Some(message) = client.receive_message(Channel::Chunk.id()) {
        let server_message = bincode::deserialize(&message).unwrap();
        chunk_messages.push(server_message);
    }
}

impl ClientMessage {
    pub fn send(&self, client: &mut RenetClient) {
        let message = bincode::serialize(self).unwrap();
        client.send_message(Channel::Reliable.id(), message);
    }
}
