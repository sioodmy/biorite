use crate::prelude::*;
use bevy_flycam::{MovementSettings, PlayerPlugin};
use crossbeam_channel::bounded;

pub struct ChunkServerPlugin;

impl Plugin for ChunkServerPlugin {
    fn build(&self, app: &mut App) {
        let (tx, rx) = bounded::<Chunk>(1000);
        let (mtx, mrx) = bounded::<Chunk>(1000);
        app.add_system(chunk_send)
            // .add_system(chunk_despawner)
            .add_system(server_chunk_spawn)
            .add_system(mesher)
            .add_plugin(PlayerPlugin)
            .add_plugin(RapierDebugRenderPlugin::default())
            .insert_resource(MovementSettings {
                sensitivity: 0.00015, // default: 0.00012
                speed: 12.0,          // default: 12.0
            })
            .insert_resource(LoadedChunks(HashMap::new()))
            .insert_resource(MeshQueueReceiver(mrx))
            .insert_resource(MeshQueueSender(mtx))
            .insert_resource(ChunkUpdaterSender(tx))
            .insert_resource(ChunkUpdaterReceiver(rx));
    }
}
