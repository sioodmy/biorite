// use bevy::prelude::StandardMaterial;

use crate::prelude::*;

pub fn render_queue(
    mut queue: ResMut<ChunkRenderQueue>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let texture_handle = asset_server.load("textures/stone.png");

    let material = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    for chunk in queue.0.iter_mut() {
        if chunk.loaded {
            trace!("chunk {:?} loaded, skipping", chunk.position);
            continue;
        }
        debug!("loading chunk {:?}", chunk.position);

        let greedy_mesh = greedy_mesh(&mut meshes, chunk.blocks);

        // spawning chunk
        commands.spawn(PbrBundle {
            mesh: greedy_mesh,
            material: material.clone(),
            transform: Transform::from_xyz(
                chunk.position.x as f32 * 16.0,
                chunk.position.y as f32 * 16.0,
                chunk.position.z as f32 * 16.0,
            ),
            ..Default::default()
        });
        chunk.loaded = true;
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
