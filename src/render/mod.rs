pub use crate::prelude::*;
use bevy::utils::HashMap;
pub use bevy_atmosphere::prelude::*;
pub use bevy_spectator::SpectatorPlugin;

pub use self::{camera::spawn_camera, render_queue::ChunkRenderQueue};

pub mod camera;
pub mod mesh;
pub mod render_queue;

pub use camera::*;
pub use mesh::*;
pub use render_queue::*;

pub struct RenderClientPlugin;
impl Plugin for RenderClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(render_queue::render_queue)
            .add_startup_system(spawn_camera)
            .add_plugin(AtmospherePlugin)
            .add_plugin(RenderQueuePlugin)
            .add_plugin(SpectatorPlugin)
            .insert_resource(Msaa { samples: 4 })
            .insert_resource(RenderDistance(HashMap::new()))
            .insert_resource(ChunkRenderQueue(vec![Chunk::default()]));
    }
}
