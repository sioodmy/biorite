use crate::prelude::*;

#[derive(Debug, Default, Deref, DerefMut, Resource)]
pub struct CurrentClientMessages(pub Vec<ServerMessage>);

#[derive(Default, Deref, DerefMut, Resource)]
pub struct CurrentClientChunkMessages(pub Vec<ServerChunkMessage>);

#[derive(Debug, Serialize, Deserialize, Resource)]
pub enum ServerChunkMessage {
    /// Multiple chunks when, for example spawn chunks
    ChunkBatch(Vec<CompressedChunk>),
    // TODO: send spawn chunks and some stuff
    Init {
        player_ids: Vec<u64>,
    },
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
