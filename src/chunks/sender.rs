use crate::prelude::*;
use rayon::prelude::*;

use std::sync::mpsc::channel;

pub fn chunk_send(
    mut server: ResMut<RenetServer>,
    mut commands: Commands,
    mut loaded: ResMut<LoadedChunks>,
    mut lobby: ResMut<Lobby>,
    update_sender: Res<ChunkUpdaterSender>,
    msg: Res<CurrentServerMessages>,
    query: Query<&mut GlobalTransform, Changed<PlayerInput>>,
) {
    for (id, message) in msg.iter() {
        if let ClientMessage::RequestChunk(vec) = message {
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
