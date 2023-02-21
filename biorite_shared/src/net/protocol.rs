use super::data_types::*;
pub use bevy::{prelude::*, utils::HashMap};
pub use bevy_renet::{renet::*, *};
use biorite_generator::{blocks::BlockType, chunk::CompressedChunk};

use serde::{Deserialize, Serialize};

pub const PROTOCOL_ID: u64 = 1;
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

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    Ping,
    PlayerInput(PlayerInput),
    // TODO: Implement blockabort packet
    BreakBlock(IVec3),
    PlaceBlock { pos: IVec3, block: BlockType },

    // RequestChunkBox([IVec3; 4]),
    RequestChunk(Vec<IVec3>),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    Pong(ServerInfo),
    PlayerSpawn(u64),
    PlayerDespawn(u64),
    EntitySync(HashMap<u64, [f32; 3]>),
    BlockDelta { pos: IVec3, block: BlockType },
}

#[derive(Debug, Serialize, Deserialize, Resource, Clone)]
pub enum ServerChunkMessage {
    /// Multiple chunks when, for example spawn chunks
    ChunkBatch(Vec<CompressedChunk>),
    // TODO: send spawn chunks and some stuff
    Init {
        player_ids: Vec<u64>,
    },
}

#[derive(Deserialize, Serialize)]
pub struct UserData(pub String);

#[allow(non_camel_case_types)]
type NETCODE_USER_DATA = [u8; NETCODE_USER_DATA_BYTES];

impl UserData {
    pub fn to_netcode_user_data(&self) -> NETCODE_USER_DATA {
        let mut user_data = [0u8; NETCODE_USER_DATA_BYTES];
        if self.0.len() > NETCODE_USER_DATA_BYTES - 8 {
            panic!("Username is too big");
        }
        user_data[0..8].copy_from_slice(&(self.0.len() as u64).to_le_bytes());
        user_data[8..self.0.len() + 8].copy_from_slice(self.0.as_bytes());

        user_data
    }

    pub fn from_user_data(user_data: &NETCODE_USER_DATA) -> Self {
        let mut buffer = [0u8; 8];
        buffer.copy_from_slice(&user_data[0..8]);
        let mut len = u64::from_le_bytes(buffer) as usize;
        len = len.min(NETCODE_USER_DATA_BYTES - 8);
        let data = user_data[8..len + 8].to_vec();
        let username = String::from_utf8(data).unwrap();
        Self(username)
    }
}

impl ClientMessage {
    pub fn send(&self, client: &mut RenetClient) {
        let message = bincode::serialize(self).unwrap();
        client.send_message(Channel::Reliable.id(), message);
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

impl ServerChunkMessage {
    pub fn send(&self, server: &mut RenetServer, id: u64) {
        let message = bincode::serialize(self).unwrap();
        debug!("Sending message");
        server.send_message(id, Channel::Chunk.id(), message);
    }
}
