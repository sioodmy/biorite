use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, VertexAttributeValues},
        render_resource::PrimitiveTopology,
    },
    tasks::{AsyncComputeTaskPool, Task},
};

use biorite_generator::{blocks::*, chunk::*, ConstShape, MeshQueueReceiver};

use block_mesh::{greedy_quads, GreedyQuadsBuffer, RIGHT_HANDED_Y_UP_CONFIG};

#[derive(Component)]
pub struct MeshTask(pub Task<Option<MeshedCollider>>);

/// Stores data required for spawning chunk entity
pub struct MeshedCollider {
    pub mesh: Mesh,
    pub chunk: Chunk,
    pub pos: IVec3,
    // _headless: bool,
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
            greedy_mesh(queued.chunk.blocks).map(|mesh| MeshedCollider {
                pos: queued.chunk.position,
                mesh,
                chunk: queued.chunk,
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
pub fn greedy_mesh(voxels: [BlockType; ChunkShape::USIZE]) -> Option<Mesh> {
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
    render_mesh.set_indices(Some(Indices::U32(indices.clone())));

    Some(render_mesh)
}
