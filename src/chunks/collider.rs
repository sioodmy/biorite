use crate::prelude::*;

use bevy::render::{
    mesh::{Indices, VertexAttributeValues},
    render_resource::PrimitiveTopology,
};
use block_mesh::{
    ndshape::ConstShape, visible_block_faces, UnitQuadBuffer,
    RIGHT_HANDED_Y_UP_CONFIG,
};

pub fn collider_mesh(
    meshes: &mut Assets<Mesh>,
    voxels: [BlockType; ChunkShape::SIZE as usize],
) -> bevy::prelude::Mesh {
    let mut buffer = UnitQuadBuffer::new();

    let faces = RIGHT_HANDED_Y_UP_CONFIG.faces;
    visible_block_faces(
        &voxels,
        &ChunkShape {},
        [0; 3],
        [CHUNK_DIM + 1; 3],
        &faces,
        &mut buffer,
    );

    info!("Generated {} quads", &buffer.num_quads());
    let num_indices = buffer.num_quads() * 6;
    let num_vertices = buffer.num_quads() * 4;
    let mut indices = Vec::with_capacity(num_indices);
    let mut positions = Vec::with_capacity(num_vertices);
    let mut normals = Vec::with_capacity(num_vertices);
    for (group, face) in buffer.groups.into_iter().zip(faces.into_iter()) {
        for quad in group.into_iter() {
            indices.extend_from_slice(
                &face.quad_mesh_indices(positions.len() as u32),
            );
            positions.extend_from_slice(
                &face.quad_mesh_positions(&quad.into(), 1.0),
            );
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

    render_mesh
}
