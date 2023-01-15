pub use self::camera::spawn_camera;
pub use crate::prelude::*;

pub use bevy_atmosphere::prelude::*;
pub use bevy_spectator::SpectatorPlugin;
use crossbeam_channel::bounded;

pub mod camera;
pub mod material;
pub mod mesh;

pub use camera::*;
pub use material::*;
pub use mesh::*;

pub struct RenderClientPlugin;
impl Plugin for RenderClientPlugin {
    fn build(&self, app: &mut App) {
        let (tx, rx) = bounded::<MeshedChunk>(500);
        app.add_plugin(MaterialPlugin::<ArrayTextureMaterial>::default())
            .add_startup_system(load_chunk_texture)
            .add_system(create_array_texture)
            .add_system_set(
                SystemSet::on_enter(AppState::InGame)
                    .with_system(spawn_camera)
                    .with_system(spawn_light),
            )
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
                    .with_system(mouse_movement)
                    .with_system(cursor_grab_system)
                    .with_system(chunk_spawner)
                    .with_system(chunk_despawner),
            )
            .add_plugin(AtmospherePlugin)
            .insert_resource(MeshChunkReceiver(rx))
            .insert_resource(MeshChunkSender(tx))
            .insert_resource(Msaa { samples: 4 });
    }
}
