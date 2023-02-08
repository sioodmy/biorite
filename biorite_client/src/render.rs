use bevy::prelude::*;

use crate::{camera::*, material::*, mesh::*, net::*, raycast::*};
use bevy::utils::HashMap;
use bevy_atmosphere::prelude::AtmospherePlugin;
use bevy_mod_raycast::DefaultRaycastingPlugin;
use biorite_generator::{
    chunk::{LoadedChunks, *},
    ConstShape, MeshQueueReceiver, MeshQueueSender, QueuedChunk,
};
use biorite_shared::net::protocol::*;
use crossbeam_channel::bounded;

use crate::raycast::intersection;

pub fn client_block_updates(
    msg: Res<CurrentClientMessages>,
    mut chunks: ResMut<LoadedChunks>,
    mesh_queue: ResMut<MeshQueueSender>,
) {
    for message in msg.iter() {
        if let ServerMessage::BlockDelta { pos, block } = message {
            info!("Got block delta at {:?} {:?}", pos, block);
            // TODO: rewrite all of this shit
            // Chunk coords
            let x = pos.x.div_euclid(CHUNK_DIM as i32);
            let y = pos.y.div_euclid(CHUNK_DIM as i32);
            let z = pos.z.div_euclid(CHUNK_DIM as i32);

            let r_x = pos.x.rem_euclid(CHUNK_DIM as i32) + 1;
            let r_y = pos.y.rem_euclid(CHUNK_DIM as i32) + 1;
            let r_z = pos.z.rem_euclid(CHUNK_DIM as i32) + 1;

            chunks.0.entry(IVec3::new(x, y, z)).and_modify(|e| {
                e.chunk.blocks[ChunkShape::linearize([
                    r_x.try_into().unwrap(),
                    r_y.try_into().unwrap(),
                    r_z.try_into().unwrap(),
                ]) as usize] = *block;
                mesh_queue
                    .0
                    .send(QueuedChunk {
                        chunk: e.chunk,
                        is_new: false,
                    })
                    .unwrap();
            });
        };
    }
}

pub struct RenderClientPlugin;
impl Plugin for RenderClientPlugin {
    fn build(&self, app: &mut App) {
        let (tx, rx) = bounded::<QueuedChunk>(1000);
        app.add_plugin(MaterialPlugin::<ArrayTextureMaterial>::default())
            .add_startup_system(load_chunk_texture)
            .add_plugin(DefaultRaycastingPlugin::<ChunkRaycast>::default())
            .add_system(create_array_texture)
            .insert_resource(MeshQueueReceiver(rx))
            .insert_resource(MeshQueueSender(tx))
            .insert_resource(Hotbar::debug())
            .insert_resource(HoldingItem(None))
            .insert_resource(LoadedChunks(HashMap::new()))
            .add_startup_system(spawn_camera)
            .add_startup_system(crosshair)
            .add_system(mouse_movement)
            .add_system(mesher)
            .add_system(client_block_updates)
            .add_system(hotbar_prototype)
            .add_system(holding_item)
            .add_system(cursor_grab_system)
            .add_system(chunk_renderer)
            .add_system(intersection) // .add_system(client_chunk_despawner),
            .add_plugin(AtmospherePlugin)
            .insert_resource(Msaa { samples: 4 });
    }
}
