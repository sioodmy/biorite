use crate::prelude::*;

pub struct ChunkServerPlugin;

impl Plugin for ChunkServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(chunk_sender);
    }
}
