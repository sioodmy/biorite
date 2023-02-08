use bevy::prelude::*;
use biorite_shared::net::protocol::*;

#[derive(Debug, Default, Deref, DerefMut, Resource)]
pub struct CurrentClientMessages(pub Vec<ServerMessage>);

#[derive(Default, Deref, DerefMut, Resource, Clone)]
pub struct CurrentClientChunkMessages(pub Vec<ServerChunkMessage>);

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
