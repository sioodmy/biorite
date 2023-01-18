use bevy::{
    render::{
        mesh::{Indices, VertexAttributeValues},
        render_resource::PrimitiveTopology,
    },
    tasks::{AsyncComputeTaskPool, Task},
};
use bevy_easings::*;
use block_mesh::{
    greedy_quads, ndshape::ConstShape, GreedyQuadsBuffer,
    RIGHT_HANDED_Y_UP_CONFIG,
};
use futures_lite::future;

use crate::prelude::*;

#[derive(Resource, Deref, DerefMut)]
pub struct MeshQueue(pub Vec<Chunk>);

#[derive(Component)]
pub struct MeshTask(Task<Option<MeshedChunk>>);

/// Stores data required for spawning chunk entity
pub struct MeshedChunk {
    mesh: Mesh,
    chunk: Chunk,
    pos: IVec3,
}

/// Once mesh is generated, apply it to the chunk entity
pub fn chunk_renderer(
    mut commands: Commands,
    mut transform_tasks: Query<(Entity, &mut MeshTask)>,
    mut meshes: ResMut<Assets<Mesh>>,
    loading_texture: Res<LoadingTexture>,
    mut loaded_chunks: ResMut<LoadedChunks>,
) {
    for (entity, mut task) in &mut transform_tasks {
        if let Some(mesh) = future::block_on(future::poll_once(&mut task.0)) {
            if let Some(meshed_chunk) = mesh {
                let chunk_entity = commands
                    .entity(entity)
                    .insert(MaterialMeshBundle {
                        mesh: meshes.add(meshed_chunk.mesh),
                        material: loading_texture.material.clone(),
                        ..Default::default()
                    })
                    .insert(
                        Transform::from_xyz(
                            meshed_chunk.pos.x as f32 * CHUNK_DIM as f32 ,
                            meshed_chunk.pos.y as f32 * CHUNK_DIM as f32 - 15.0,
                            meshed_chunk.pos.z as f32 * CHUNK_DIM as f32,
                        )
                        .ease_to(
                            Transform::from_xyz(
                                meshed_chunk.pos.x as f32 * CHUNK_DIM as f32,
                                meshed_chunk.pos.y as f32 * CHUNK_DIM as f32,
                                meshed_chunk.pos.z as f32 * CHUNK_DIM as f32,
                            ),
                            EaseFunction::QuadraticIn,
                            EasingType::Once {
                                duration: std::time::Duration::from_millis(350),
                            },
                        ),
                    )
                    .insert(RaycastMesh::<MyRaycastSet>::default())
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

pub fn chunk_despawner(
    mut commands: Commands,
    mut loaded_chunks: ResMut<LoadedChunks>,
    player_query: Query<&GlobalTransform, With<Player>>,
) {
    // List of chunks that we actually need
    let _span = info_span!("unloading_chunks", name = "unloading_chunks").entered();
    let mut relevant = Vec::new();
    for player in player_query.iter() {
        let player_coords = player.translation().as_ivec3();
        // Nearest chunk origin
        let no = !IVec3::splat((CHUNK_DIM - 1) as i32) & player_coords;
        let chunk_x = no.x / CHUNK_DIM as i32;
        let chunk_y = no.y / CHUNK_DIM as i32;
        let chunk_z = no.z / CHUNK_DIM as i32;
        for x in (chunk_x - RENDER_DISTANCE as i32)
            ..=(chunk_x + RENDER_DISTANCE as i32)
        {
            for y in (chunk_y - RENDER_DISTANCE as i32)
                ..=(chunk_y + RENDER_DISTANCE as i32)
            {
                for z in (chunk_z - RENDER_DISTANCE as i32)
                    ..=(chunk_z + RENDER_DISTANCE as i32)
                {
                    let chunk = IVec3::new(x as i32, y as i32, z as i32);
                    relevant.push(chunk);
                }
            }
        }
    }

    loaded_chunks
        .0
        .drain_filter(|pos, _| !relevant.contains(pos))
        .for_each(|(_, entry)| commands.entity(entry.entity).despawn());
    loaded_chunks.0.shrink_to_fit();
}

/// Spawns `MeshTask` task to parallelize greedy meshing, because it's quite
/// expensive operation
pub fn mesher(mut mesh_queue: ResMut<MeshQueue>, mut commands: Commands) {
    let thread_pool = AsyncComputeTaskPool::get();
    // Limit how many chunks can be meshed per frame to avoid lag spikes
    let limit = usize::min(mesh_queue.0.len(), 3);
    for chunk in mesh_queue.0.drain(..limit) {
        let task = thread_pool.spawn(async move {
            debug!("meshing, {:?}", chunk.position);
            greedy_mesh(chunk.blocks).map(|mesh| MeshedChunk {
                pos: chunk.position,
                mesh,
                chunk,
            })
        });
        commands.spawn(MeshTask(task));
    }
}

/// Optimizes chunk mesh, by reducing number of vertices gpu has to render
/// See https://0fps.net/2012/06/30/meshing-in-a-minecraft-game/
pub fn greedy_mesh(
    voxels: [BlockType; ChunkShape::SIZE as usize],
) -> Option<Mesh> {
    let _span = info_span!("greedy_mesh", name = "greedy_mesh").entered();

    let mut buffer = GreedyQuadsBuffer::new(voxels.len());
    let faces = RIGHT_HANDED_Y_UP_CONFIG.faces;

    greedy_quads(
        &voxels,
        &ChunkShape {},
        [0; 3],
        [CHUNK_DIM + 1; 3],
        &RIGHT_HANDED_Y_UP_CONFIG.faces,
        &mut buffer,
    );

    // This means that chunk is 100% air
    if buffer.quads.num_quads() == 0 {
        return None;
    }

    let num_indices = buffer.quads.num_quads() * 6;

    let num_vertices = buffer.quads.num_quads() * 4;

    let mut indices = Vec::with_capacity(num_indices);

    let mut positions = Vec::with_capacity(num_vertices);

    let mut normals = Vec::with_capacity(num_vertices);

    let mut tex_coords = Vec::with_capacity(num_vertices);

    let mut indexes = Vec::with_capacity(num_vertices);
    let mut lights = Vec::with_capacity(num_vertices);

    for (group, face) in buffer.quads.groups.into_iter().zip(faces.into_iter())
    {
        for quad in group.into_iter() {
            let normal = face.quad_mesh_normals();
            let face_indices = face.quad_mesh_indices(positions.len() as u32);
            let face_positions = face.quad_mesh_positions(&quad, 1.0);
            let face_index: Vec<_> = face_positions
                .iter()
                .map(|_| {
                    let i = ChunkShape::linearize(quad.minimum.map(|v| v));
                    let voxel = voxels[i as usize];
                    let dir = BlockFace::from_normal(normal);
                    voxel.get_texture().from_direction(dir)
                })
                .collect();

            indices.extend_from_slice(&face_indices);

            positions.extend_from_slice(&face.quad_mesh_positions(&quad, 1.0));
            indexes.extend_from_slice(&face_index);

            let light: Vec<_> = face_positions
                .iter()
                .map(|_| {
                    if normal
                        == [
                            [0.0, 1.0, 0.0],
                            [0.0, 1.0, 0.0],
                            [0.0, 1.0, 0.0],
                            [0.0, 1.0, 0.0],
                        ]
                    {
                        1.0
                    } else {
                        0.6
                    }
                })
                .collect();
            normals.extend_from_slice(&normal);
            lights.extend_from_slice(&light);
            tex_coords.extend_from_slice(&face.tex_coords(
                RIGHT_HANDED_Y_UP_CONFIG.u_flip_face,
                true,
                &quad,
            ));
        }
    }

    let mut render_mesh = Mesh::new(PrimitiveTopology::TriangleList);

    render_mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float32x3(positions),
    );

    render_mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        VertexAttributeValues::Float32x3(normals),
    );

    render_mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        VertexAttributeValues::Float32x2(tex_coords),
        // VertexAttributeValues::Float32x2(vec![[0.0; 2]; num_vertices]),
    );

    render_mesh.insert_attribute(
        ArrayTextureMaterial::ATTRIBUTE_TEXTURE_INDEX,
        VertexAttributeValues::Sint32(indexes),
    );
    render_mesh.insert_attribute(
        ArrayTextureMaterial::ATTRIBUTE_LIGHT,
        VertexAttributeValues::Float32(lights),
    );

    render_mesh.set_indices(Some(Indices::U32(indices.clone())));

    Some(render_mesh)
}
