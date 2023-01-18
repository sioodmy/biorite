use crate::prelude::*;
use crossbeam_channel::{Receiver, Sender};

#[derive(Resource)]
pub struct ChunkUpdaterSender(pub Sender<Chunk>);

#[derive(Resource)]
pub struct ChunkUpdaterReceiver(pub Receiver<Chunk>);

// pub fn server_chunk_spawn(
//     mut commands: Commands,
//     rx: Res<ChunkUpdaterReceiver>,
//     mut loaded: Query<&Chunk>,
// ) {
//     rx.0.try_iter().for_each(|chunk| {
//         debug!("Spawning chunk {:?}", chunk.position);
//         commands.spawn(chunk);
//     });
// }
