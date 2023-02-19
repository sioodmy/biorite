use biorite_shared::net::{data_types::*, protocol::*};

use super::util::*;

/// Slippage between predicted player movement and sync packets
fn calculate_delta(predicted: [f32; 3], translation: [f32; 3]) -> f32 {
    predicted
        .iter()
        .zip(translation.iter())
        .map(|(p, t)| (p - t).abs())
        .max_by(|x, y| x.partial_cmp(y).unwrap())
        .unwrap()
}

pub fn entity_sync(
    lobby: ResMut<Lobby>,
    mut commands: Commands,
    messages: Res<CurrentClientMessages>,
    query: Query<&Transform, With<Player>>,
    client: Res<RenetClient>,
) {
    for message in messages.iter() {
        #[allow(irrefutable_let_patterns)]
        // who tf wrote all of this shit (couldn't be me)
        if let ServerMessage::EntitySync(sync) = message {
            for (player_id, translation) in sync.iter() {
                if let Some(player_entity) = lobby.players.get(player_id) {
                    // Self prediciton
                    if *player_id == client.client_id() {
                        if let Ok(predicted) = query.get(*player_entity) {
                            let _delta = calculate_delta(
                                predicted.translation.into(),
                                *translation,
                            );

                            // Do not bother if our prediction is correct
                            // Rollback
                            // if delta > 3.
                        }
                    } else {
                        if let Ok(old_pos) = query.get(*player_entity) {
                            let old: [f32; 3] = old_pos.translation.into();
                            let diff: Vec<f32> = old
                                .iter()
                                .zip(translation.iter())
                                .map(|(p, t)| (p - t).abs())
                                .collect();
                            dbg!(diff);
                        }

                        let transform = Transform {
                            translation: (*translation).into(),
                            ..Default::default()
                        };
                        commands.entity(*player_entity).insert(transform);
                    }
                }
            }
        }
    }
}
