use crate::prelude::*;
use crossbeam_channel::bounded;

pub struct ChunkServerPlugin;

impl Plugin for ChunkServerPlugin {
    fn build(&self, app: &mut App) {
        let (tx, rx) = bounded::<Chunk>(1000);
        let (mtx, mrx) = bounded::<QueuedChunk>(1000);
        app.add_system(chunk_send)
            // .add_system(chunk_despawner)
            .add_system(server_chunk_spawn)
            .add_system(mesher)
            .insert_resource(LoadedChunks(HashMap::new()))
            .insert_resource(MeshQueueReceiver(mrx))
            .insert_resource(MeshQueueSender(mtx))
            .insert_resource(ChunkUpdaterSender(tx))
            .insert_resource(ChunkUpdaterReceiver(rx));
    }
}
