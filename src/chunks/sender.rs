use crate::prelude::*;

use std::sync::mpsc::channel;

/// Send chunks to the client
pub fn chunk_send(
    mut server: ResMut<RenetServer>,
    msg: Res<CurrentServerMessages>,
    // query: Query<&mut GlobalTransform, Changed<PlayerInput>>,
) {
    for (id, message) in msg.iter() {
        if let ClientMessage::RequestChunk(vec) = message {
            // TODO: Validate each request
            let (tx, rx) = channel();
            for pos in vec.iter() {
                info!("Generating chunk at {:?}", pos);
                let sender = tx.clone();
                sender
                    .clone()
                    .send(chunk_generator(pos).compress())
                    .unwrap();
            }
            let vector: Vec<CompressedChunk> = rx.try_iter().collect();
            ServerChunkMessage::ChunkBatch(vector).send(&mut server, *id);
        }
    }
}
