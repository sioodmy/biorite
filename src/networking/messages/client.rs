use crate::prelude::*;

#[derive(Debug, Default, Deref, DerefMut, Resource)]
pub struct CurrentServerMessages(pub Vec<(u64, ClientMessage)>);

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    Ping,
    PlayerInput(PlayerInput),
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
