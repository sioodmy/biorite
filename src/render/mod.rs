pub use self::camera::spawn_camera;
pub use crate::prelude::*;

pub use bevy_atmosphere::prelude::*;
pub use bevy_spectator::SpectatorPlugin;
use crossbeam_channel::bounded;
use smooth_bevy_cameras::controllers::fps::FpsCameraPlugin;

pub mod camera;
pub mod material;
pub mod mesh;

pub use camera::*;
pub use material::*;
pub use mesh::*;

pub struct RenderClientPlugin;
impl Plugin for RenderClientPlugin {
    fn build(&self, app: &mut App) {
        let (tx, rx) = bounded::<MeshedChunk>(200);
        app.add_plugin(MaterialPlugin::<ArrayTextureMaterial>::default())
            .add_startup_system(spawn_camera)
            .add_startup_system(load_chunk_texture)
            .add_startup_system(spawn_light)
            .add_system(create_array_texture)
            .add_system(mouse_movement)
            .add_system(cursor_grab_system)
            .add_system(chunk_spawner)
            .add_plugin(AtmospherePlugin)
            // .add_plugin(RenderQueuePlugin)
            .add_plugin(SpectatorPlugin)
            .add_plugin(FpsCameraPlugin::default())
            .insert_resource(MeshChunkReceiver(rx))
            .insert_resource(MeshChunkSender(tx))
            .insert_resource(Msaa { samples: 4 });
    }
}
