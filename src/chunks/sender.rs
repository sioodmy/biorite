use crate::prelude::*;

pub fn chunk_sender(
    messages: Res<CurrentServerMessages>,
    mut server: ResMut<RenetServer>,
    mut queued_requests: Local<Vec<(u64, IVec3)>>,
    mut noise: ResMut<NoiseResource>,
) {
    queued_requests.retain(|(id, position)| {
        if server.can_send_message(*id, Channel::Chunk.id()) {
            for (client_id, message) in messages.iter() {
                debug!("Sending from queue");
                let chunk = chunk_generator(*position, &noise.0).compress();
                debug!("Packet size {}", chunk.len() * 8);
                ServerChunkMessage::Chunk(chunk).send(&mut server, *client_id);
                return false;
            }
        }
        true
    });

    for message in messages.iter() {
        if let (id, ClientMessage::RequestChunk(position)) = message {
            if server.can_send_message(*id, Channel::Chunk.id()) {
                debug!("Sending Chunk! {}", position);
                let chunk = chunk_generator(*position, &noise.0).compress();
                ServerChunkMessage::Chunk(chunk).send(&mut server, *id);
            } else {
                queued_requests.push((*id, *position));
            }
        }
    }
}
