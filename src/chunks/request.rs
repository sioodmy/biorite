use crate::prelude::*;

pub fn chunk_receiver(
    tx: ResMut<MeshChunkSender>,
    chunk_messages: Res<CurrentClientChunkMessages>,
) {
    for message in chunk_messages.iter() {
        if let ServerChunkMessage::ChunkBatch(compressed_batch) = message {
            debug!("Chunk batch");
            mesher(compressed_batch.clone(), tx.0.clone());
        }
    }
}
