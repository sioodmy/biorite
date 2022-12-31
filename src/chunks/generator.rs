use crate::prelude::*;

pub fn chunk_sender(
    messages: Res<CurrentServerMessages>,
    mut server: ResMut<RenetServer>,
    mut queued_requests: Local<Vec<(u64, IVec3)>>,
) {
    queued_requests.retain(|(id, pos)| {
        if server.can_send_message(*id, Channel::Chunk.id()) {
            for (client_id, message) in messages.iter() {
                if let ClientMessage::RequestChunk(position) = message {
                    debug!("Client {} requested chunk at {:?}", client_id, position);
                    let chunk = chunk_generator(*position).compress();
                    debug!("Packet size {}", chunk.len() * 8);
                    ServerChunkMessage::Chunk(chunk).send(&mut server, *client_id);
                    return false;
                }
            }
        }
        true
    });
}

pub fn request_handler(
    messages: Res<CurrentServerMessages>,
    mut server: ResMut<RenetServer>,
    mut queued_requests: Local<Vec<(u64, IVec3)>>,
) {
    for message in messages.iter() {
        if let (id, ClientMessage::RequestChunk(position)) = message {
            if server.can_send_message(*id, Channel::Chunk.id()) {
                debug!("Sending Chunk! {}", position);
                let chunk = chunk_generator(*position).compress();
                ServerChunkMessage::Chunk(chunk).send(&mut server, *id);
            } else {
                debug!("Skipping chunk {}", position);
                queued_requests.push((*id, *position));
            }
        }
    }
}

pub fn chunk_generator(position: IVec3) -> Chunk {
    // placeholder for propper chunk generation
    let mut blocks: [BlockType; ChunkShape::SIZE as usize] = [AIR; ChunkShape::SIZE as usize];
    for x in 1..17 {
        for z in 1..17 {
            for y in 1..12 {
                let i = ChunkShape::linearize([x, y, z]);
                blocks[i as usize] = STONE;
            }
            for y in 12..17 {
                let i = ChunkShape::linearize([x, y, z]);
                blocks[i as usize] = DIRT;
            }
        }
    }
    Chunk {
        position,
        blocks,
        ..Default::default()
    }
}
