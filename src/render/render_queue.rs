use crate::prelude::*;
use bevy::{prelude::*, tasks::AsyncComputeTaskPool};
use crossbeam_channel::{bounded, Sender};

pub fn render_queue(
    mut queue: ResMut<ChunkRenderQueue>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut renderd: ResMut<RenderDistance>,
) {
    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        alpha_mode: AlphaMode::Blend,
        unlit: false,
        ..default()
    });

    for chunk in queue.0.iter_mut() {
        if chunk.loaded {
            continue;
        }
        debug!("loading chunk {:?}", chunk.position);

        let greedy_mesh = greedy_mesh(&mut meshes, chunk.blocks);

        let chunk_entity = commands
            .spawn(PbrBundle {
                mesh: greedy_mesh,
                material: material.clone(),
                transform: Transform::from_xyz(
                    chunk.position.x as f32 * CHUNK_DIM as f32,
                    chunk.position.y as f32 * CHUNK_DIM as f32,
                    chunk.position.z as f32 * CHUNK_DIM as f32,
                ),
                ..Default::default()
            })
            .id();

        chunk.loaded = true;
        renderd.0.insert(chunk.position, (true, chunk_entity));
    }
}

#[derive(Debug, Resource)]
pub struct ChunkRenderQueue(pub Vec<Chunk>);

pub struct RenderQueuePlugin;

impl Plugin for RenderQueuePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ChunkRenderQueue(vec![Chunk::default()]))
            .add_system(render_queue);
    }
}
