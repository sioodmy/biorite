use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_renet::renet::*;
use biorite_generator::{
    chunk::{ChunkEntry, LoadedChunks, CHUNK_DIM},
    *,
};
use biorite_shared::net::{
    data_types::{Lobby, Player},
    protocol::*,
};
use crossbeam_channel::bounded;
use futures_lite::future;

use crate::{
    collider::{mesher, MeshTask},
    net::CurrentServerMessages,
};

// pub struct Pending(HashMap<u64, IVec3>);

/// Send chunks to the client
pub fn chunk_send(
    mut server: ResMut<RenetServer>,
    mtx: ResMut<MeshQueueSender>,
    msg: Res<CurrentServerMessages>,
    lobby: Res<Lobby>,
    query: Query<&GlobalTransform, With<Player>>,
    mut save: ResMut<SaveFile>,
    // query: Query<&mut GlobalTransform, Changed<PlayerInput>>,
) {
    for (id, message) in msg.iter() {
        if let ClientMessage::RequestChunk(vec) = message {
            // TODO: Validate each request
            // TODO: Queue chunk pending requests so that server sends chunks
            // evenly
            if vec.len() > 512 {
                warn!("Client requested too many chunks. Disconnecting");
                server.disconnect(*id);
            }

            info!("client reqested chunk");
            let mesh_sender = mtx.0.clone();
            if let Some(player_entity) = lobby.players.get(id) {
                let _coords = query.get(*player_entity).unwrap();
                let chunks =
                    save.get_compressed_chunk(vec.to_vec(), mesh_sender);
                for batch in chunks.chunks(5) {
                    // debug!("Sending chunk batch with len {}", bat.len());
                    ServerChunkMessage::ChunkBatch(batch.to_vec())
                        .send(&mut server, *id);
                }
            }

            // let vector: Vec<CompressedChunk> = rx.try_iter().collect();
            // let vectors: Vec<Vec<CompressedChunk>> = .drain().collect();
        }
    }
}

pub fn server_chunk_spawn(
    mut commands: Commands,
    mut transform_tasks: Query<(Entity, &mut MeshTask)>,
    mut loaded_chunks: ResMut<LoadedChunks>,
) {
    for (entity, mut task) in &mut transform_tasks {
        if let Some(mesh) = future::block_on(future::poll_once(&mut task.0)) {
            if let Some(meshed_chunk) = mesh {
                info!("spawning chunk {:?}", meshed_chunk.pos);
                let chunk_entity = commands
                    .entity(entity)
                    .insert(PbrBundle {
                        transform: Transform::from_xyz(
                            meshed_chunk.pos.x as f32 * CHUNK_DIM as f32,
                            meshed_chunk.pos.y as f32 * CHUNK_DIM as f32,
                            meshed_chunk.pos.z as f32 * CHUNK_DIM as f32,
                        ),
                        ..Default::default()
                    })
                    .insert(ColliderMassProperties::Density(100000.0))
                    .insert(
                        Collider::from_bevy_mesh(
                            &meshed_chunk.mesh,
                            &ComputedColliderShape::TriMesh,
                        )
                        .unwrap(),
                    )
                    .id();
                loaded_chunks.0.insert(
                    meshed_chunk.pos,
                    ChunkEntry {
                        chunk: meshed_chunk.chunk,
                        entity: chunk_entity,
                    },
                );
            } else {
                commands.entity(entity).despawn();
            };

            // Task is complete, so remove task component from entity
            commands.entity(entity).remove::<MeshTask>();
        }
    }
}

pub struct ChunkServerPlugin;

impl Plugin for ChunkServerPlugin {
    fn build(&self, app: &mut App) {
        let (mtx, mrx) = bounded::<QueuedChunk>(1000);
        app.insert_resource(LoadedChunks(HashMap::new()))
            .insert_resource(MeshQueueReceiver(mrx))
            .insert_resource(ModifiedRegions::default())
            .insert_resource(MeshQueueSender(mtx))
            .add_system(mesher)
            .add_system(chunk_send)
            .add_system(server_chunk_spawn);
    }
}
