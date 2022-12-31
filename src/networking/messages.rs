use crate::prelude::*;
use bevy::ecs::prelude::*;
pub use bevy_renet::{renet::*, *};
pub use serde::{Deserialize, Serialize};

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
            Channel::Unreliable => UnreliableChannelConfig::default().channel_id,
            Channel::Chunk => ChunkChannelConfig::default().channel_id,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerChunkMessage {
    Chunk(CompressedChunk),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    Ping,
    RequestChunk(IVec3),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    Pong(ServerInfo),
}

impl ServerChunkMessage {
    pub fn send(&self, server: &mut RenetServer, id: u64) {
        let message = bincode::serialize(self).unwrap();
        server.send_message(id, Channel::Chunk.id(), message);
    }
}

impl ServerMessage {
    pub fn send(&self, server: &mut RenetServer, id: u64) {
        let message = bincode::serialize(self).unwrap();
        match self {
            ServerMessage::Pong(_) => server.send_message(id, Channel::Reliable.id(), message),
        }
    }
}
pub fn server_recieve_messages(
    mut server: ResMut<RenetServer>,
    mut messages: ResMut<CurrentServerMessages>,
) {
    messages.0.clear();
    let channel_id = 0;
    // Send a text message for all clients
    for client_id in server.clients_id().into_iter() {
        while let Some(message) = server.receive_message(client_id, channel_id) {
            let client_message = bincode::deserialize(&message).unwrap();
            messages.0.push((client_id, client_message));
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
    let channel_id = 0;
    while let Some(message) = client.receive_message(channel_id) {
        let server_message = bincode::deserialize(&message).unwrap();
        messages.0.push(server_message);
    }
}

impl ClientMessage {
    pub fn send(&self, client: &mut RenetClient) {
        let message = bincode::serialize(self).unwrap();
        client.send_message(Channel::Reliable.id(), message);
    }
}
// pub fn client_recieve_messages(mut client: ResMut<RenetClient>) {
//     for channel in [Channel::Reliable, Channel::Unreliable] {
//         while let Some(message) = client.receive_message(channel.id()) {}
//     }
//     while let Some(message) = client.receive_message(Channel::Chunk) {
//         let server_message = bincode::deserialize(&message).unwrap();
//         chunk_messages.push(server_message);
//     }
// }
