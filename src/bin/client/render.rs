use bevy::render::{
    mesh::{Indices, VertexAttributeValues},
    render_resource::PrimitiveTopology,
};
use block_mesh::{greedy_quads, ndshape::ConstShape, GreedyQuadsBuffer, RIGHT_HANDED_Y_UP_CONFIG};

use crate::{BlockType, *};

pub fn renderq(mut queue: ResMut<ChunkRenderQueue>) {
    trace!("{:?}", queue.0);
}

pub fn greedy_mesh(
    meshes: &mut Assets<Mesh>,
    voxels: [BlockType; ChunkShape::SIZE as usize],
) -> Handle<Mesh> {
    let mut buffer = GreedyQuadsBuffer::new(voxels.len());

    let faces = RIGHT_HANDED_Y_UP_CONFIG.faces;

    greedy_quads(
        &voxels,
        &ChunkShape {},
        [0; 3],
        [17; 3],
        &RIGHT_HANDED_Y_UP_CONFIG.faces,
        &mut buffer,
    );

    info!("Generated {} quads", &buffer.quads.num_quads());

    let num_indices = buffer.quads.num_quads() * 6;

    let num_vertices = buffer.quads.num_quads() * 4;

    let mut indices = Vec::with_capacity(num_indices);

    let mut positions = Vec::with_capacity(num_vertices);

    let mut normals = Vec::with_capacity(num_vertices);

    for (group, face) in buffer.quads.groups.into_iter().zip(faces.into_iter()) {
        for quad in group.into_iter() {
            indices.extend_from_slice(&face.quad_mesh_indices(positions.len() as u32));

            positions.extend_from_slice(&face.quad_mesh_positions(&quad, 1.0));

            normals.extend_from_slice(&face.quad_mesh_normals());
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
        VertexAttributeValues::Float32x2(vec![[0.0; 2]; num_vertices]),
    );

    render_mesh.set_indices(Some(Indices::U32(indices.clone())));

    meshes.add(render_mesh)
}
