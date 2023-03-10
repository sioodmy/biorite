use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, VertexAttributeValues},
        render_resource::PrimitiveTopology,
    },
    tasks::{AsyncComputeTaskPool, Task},
};
use bevy_mod_raycast::RaycastMesh;

use crate::{material::*, raycast::ChunkRaycast};
use bevy_easings::*;
use bevy_rapier3d::prelude::*;
use biorite_generator::{blocks::*, chunk::*, MeshQueueReceiver};
use biorite_shared::consts::*;

use block_mesh::{
    greedy_quads, ndshape::ConstShape, GreedyQuadsBuffer,
    RIGHT_HANDED_Y_UP_CONFIG,
};
use futures_lite::future;

#[derive(Component)]
pub struct MeshTask(Task<Option<MeshedChunk>>);

/// Stores data required for spawning chunk entity
pub struct MeshedChunk {
    mesh: Mesh,
    chunk: Chunk,
    pos: IVec3,
    is_new: bool,
    // _headless: bool,
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
                commands.entity(entity).insert((
                    Collider::from_bevy_mesh(
                        &meshed_chunk.mesh,
                        &ComputedColliderShape::TriMesh,
                    )
                    .unwrap(),
                    RaycastMesh::<ChunkRaycast>::default(),
                    MaterialMeshBundle {
                        mesh: meshes.add(meshed_chunk.mesh),
                        material: loading_texture.material.clone(),
                        transform: Transform::from_xyz(
                            meshed_chunk.pos.x as f32 * CHUNK_DIM as f32,
                            meshed_chunk.pos.y as f32 * CHUNK_DIM as f32,
                            meshed_chunk.pos.z as f32 * CHUNK_DIM as f32,
                        ),
                        ..Default::default()
                    },
                ));

                if meshed_chunk.is_new {
                    commands
                        .entity(entity)
                        .insert(ColliderMassProperties::Density(100000.0))
                        .insert(
                            Transform::from_xyz(
                                meshed_chunk.pos.x as f32 * CHUNK_DIM as f32,
                                meshed_chunk.pos.y as f32 * CHUNK_DIM as f32
                                    - CHUNK_ANIMATION_OFFSET,
                                meshed_chunk.pos.z as f32 * CHUNK_DIM as f32,
                            )
                            .ease_to(
                                Transform::from_xyz(
                                    meshed_chunk.pos.x as f32
                                        * CHUNK_DIM as f32,
                                    meshed_chunk.pos.y as f32
                                        * CHUNK_DIM as f32,
                                    meshed_chunk.pos.z as f32
                                        * CHUNK_DIM as f32,
                                ),
                                EaseFunction::QuadraticIn,
                                EasingType::Once {
                                    duration: std::time::Duration::from_millis(
                                        CHUNK_ANIMATION_DURATION,
                                    ),
                                },
                            ),
                        );
                }
                loaded_chunks.0.insert(
                    meshed_chunk.pos,
                    ChunkEntry {
                        chunk: meshed_chunk.chunk,
                        entity,
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

/// Spawns `MeshTask` task to parallelize greedy meshing, because it's quite
/// expensive operation
pub fn mesher(
    mesh_queue: ResMut<MeshQueueReceiver>,
    mut commands: Commands,
    loaded_chunks: ResMut<LoadedChunks>,
) {
    let thread_pool = AsyncComputeTaskPool::get();
    for queued in mesh_queue.0.try_iter() {
        let exists = loaded_chunks.0.get(&queued.chunk.position);
        let task = thread_pool.spawn(async move {
            greedy_mesh(queued.chunk.blocks).map(|mesh| MeshedChunk {
                pos: queued.chunk.position,
                mesh,
                chunk: queued.chunk,
                is_new: queued.is_new,
            })
        });

        if let Some(existing_chunk) = exists {
            commands
                .entity(existing_chunk.entity)
                .insert(MeshTask(task));
        } else {
            commands.spawn(MeshTask(task));
        }
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

    let mut boundary = [BlockType::Air; ChunkBoundary::USIZE];
    for x in 0..CHUNK_DIM {
    for y in 0..CHUNK_DIM {
    for z in 0..CHUNK_DIM {
                // println!("{}/{}/{}", x, y, z);
                boundary[ChunkBoundary::linearize([x + 1, y + 1, z + 1]) as usize] = voxels[ChunkShape::linearize([x,y,z]) as usize];

        }
        }
        
    }

    
    greedy_quads(
        &boundary,
        &ChunkBoundary {},
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
    let mut ambient_occlusion = Vec::with_capacity(num_vertices);

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
                    let i = ChunkBoundary::linearize(quad.minimum.map(|v| v));
                    let voxel = boundary[i as usize];
                    let dir = BlockFace::from_normal(normal);
                    voxel.get_texture().from_direction(dir)
                })
                .collect();

            let ao = face.quad_mesh_ao(&quad);
            for ambient in ao {
                ambient_occlusion.push(ambient as f32);
            }
            indices.extend_from_slice(&face_indices);

            positions.extend_from_slice(&face.quad_mesh_positions(&quad, 1.0));
            indexes.extend_from_slice(&face_index);

            let light: Vec<_> = face_positions.iter().map(|_| 1.0).collect();

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
    render_mesh.insert_attribute(
        ArrayTextureMaterial::ATTRIBUTE_AO,
        VertexAttributeValues::Float32(ambient_occlusion),
    );

    render_mesh.set_indices(Some(Indices::U32(indices.clone())));

    Some(render_mesh)
}
