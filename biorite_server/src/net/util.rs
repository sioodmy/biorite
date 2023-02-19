use bevy::prelude::*;
use biorite_shared::net::protocol::*;

#[derive(Debug, Default, Deref, DerefMut, Resource)]
pub struct CurrentServerMessages(pub Vec<(u64, ClientMessage)>);

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
