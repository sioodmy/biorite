pub use self::{camera::spawn_camera, render_queue::ChunkRenderQueue};
pub use crate::prelude::*;
use bevy::utils::HashMap;
pub use bevy_atmosphere::prelude::*;
pub use bevy_spectator::SpectatorPlugin;
use smooth_bevy_cameras::controllers::fps::FpsCameraPlugin;

pub mod camera;
pub mod material;
pub mod mesh;
pub mod render_queue;

pub use camera::*;
pub use material::*;
pub use mesh::*;
pub use render_queue::*;

pub struct RenderClientPlugin;
impl Plugin for RenderClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(render_queue::render_queue)
            .add_plugin(MaterialPlugin::<ArrayTextureMaterial>::default())
            .add_startup_system(spawn_camera)
            .add_startup_system(load_chunk_texture)
            .add_startup_system(spawn_light)
            .add_system(create_array_texture)
            .add_system(mouse_movement)
            .add_plugin(AtmospherePlugin)
            .add_plugin(RenderQueuePlugin)
            .add_plugin(SpectatorPlugin)
            .add_plugin(FpsCameraPlugin::default())
            .insert_resource(Msaa { samples: 4 })
            .insert_resource(RenderDistance(HashMap::new()))
            .insert_resource(ChunkRenderQueue(vec![Chunk::default()]));
    }
}
