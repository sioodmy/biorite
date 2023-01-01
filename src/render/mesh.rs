use bevy::render::color::Color;
use bevy::render::{
    mesh::{Indices, VertexAttributeValues},
    render_resource::PrimitiveTopology,
};
use block_mesh::{greedy_quads, ndshape::ConstShape, GreedyQuadsBuffer, RIGHT_HANDED_Y_UP_CONFIG};

use crate::prelude::*;

const UV_SCALE: f32 = 1.0 / CHUNK_DIM as f32;

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
        [CHUNK_DIM + 1; 3],
        &RIGHT_HANDED_Y_UP_CONFIG.faces,
        &mut buffer,
    );

    info!("Generated {} quads", &buffer.quads.num_quads());

    let num_indices = buffer.quads.num_quads() * 6;

    let num_vertices = buffer.quads.num_quads() * 4;

    let mut indices = Vec::with_capacity(num_indices);

    let mut positions = Vec::with_capacity(num_vertices);

    let mut normals = Vec::with_capacity(num_vertices);

    let mut tex_coords = Vec::with_capacity(num_vertices);

    let mut colors = Vec::with_capacity(num_vertices);

    for (group, face) in buffer.quads.groups.into_iter().zip(faces.into_iter()) {
        for quad in group.into_iter() {
            let face_indices = face.quad_mesh_indices(positions.len() as u32);
            let face_positions = face.quad_mesh_positions(&quad, 1.0);
            let face_colors: Vec<_> = face_positions
                .iter()
                .map(|_| {
                    let i = ChunkShape::linearize(quad.minimum.map(|v| v).into());
                    let voxel = voxels[i as usize];
                    match voxel.0 {
                        0 => unreachable!(),
                        1 => {
                            let c = color(voxel.0);
                            [0.9, 0.2, 0.1, 1.0]
                        }
                        _ => {
                            let c = color(voxel.0);
                            [0.0, 0.0, 0.9, 1.0]
                        }
                    }
                })
                .collect();

            indices.extend_from_slice(&face.quad_mesh_indices(positions.len() as u32));

            positions.extend_from_slice(&face.quad_mesh_positions(&quad, 1.0));
            colors.extend_from_slice(&face_colors);

            normals.extend_from_slice(&face.quad_mesh_normals());
            tex_coords.extend_from_slice(&face.tex_coords(
                RIGHT_HANDED_Y_UP_CONFIG.u_flip_face,
                true,
                &quad,
            ));
        }
    }

    for uv in tex_coords.iter_mut() {
        for c in uv.iter_mut() {
            *c *= UV_SCALE;
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
    );

    render_mesh.insert_attribute(
        Mesh::ATTRIBUTE_COLOR,
        VertexAttributeValues::Float32x4(colors),
    );

    render_mesh.set_indices(Some(Indices::U32(indices.clone())));

    meshes.add(render_mesh)
}

fn color(level: u16) -> Color {
    let c = if level < 22 {
        let g = colorgrad::CustomGradient::new()
            .colors(&[
                colorgrad::Color::from_rgba8(0, 0, 30, 255),
                colorgrad::Color::from_rgba8(30, 30, 200, 255),
            ])
            .build()
            .unwrap();
        g.at(level as f64 / 22.0)
    } else if level >= 22 && level <= 24 {
        let g = colorgrad::CustomGradient::new()
            .colors(&[
                colorgrad::Color::from_rgba8(195, 182, 153, 255),
                colorgrad::Color::from_rgba8(190, 153, 72, 255),
            ])
            .build()
            .unwrap();
        g.at((level as f64 - 22.0) / 2.0)
    } else if level > 24 && level <= 29 {
        let g = colorgrad::CustomGradient::new()
            .colors(&[
                colorgrad::Color::from_rgba8(0, 114, 0, 255),
                colorgrad::Color::from_rgba8(0, 20, 0, 255),
            ])
            .build()
            .unwrap();
        g.at((level as f64 - 25.0) / 4.0)
    } else if level <= 50 {
        let g = colorgrad::CustomGradient::new()
            .colors(&[
                colorgrad::Color::from_rgba8(207, 105, 17, 255),
                colorgrad::Color::from_rgba8(105, 52, 5, 255),
            ])
            .build()
            .unwrap();
        g.at((level as f64 - 30.0) / 20.0)
    } else {
        let g = colorgrad::CustomGradient::new()
            .colors(&[
                colorgrad::Color::from_rgba8(69, 64, 59, 255),
                colorgrad::Color::from_rgba8(30, 30, 30, 255),
            ])
            .build()
            .unwrap();
        g.at((level as f64 - 50.0) / 14.0)
    };
    Color::rgba(c.r as f32, c.g as f32, c.b as f32, 1.0)
}
