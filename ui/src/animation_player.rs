use std::collections::HashMap;

use eframe::egui;
use engine_core::EngineCore;
use state::{schema::playable_schema::PlayableSchema, utils::get_current_time_ms};

/// Processes the animations.
/// Returns whether render is needed and whether the game state should advance.
pub fn process_animations(
    ui: &mut egui::Ui,
    playable: &mut PlayableSchema,
    animation_indices: &mut HashMap<String, usize>,
    animation_timestamps_ms: &mut HashMap<String, u64>,
) -> (bool, bool) {
    let mut request_repaint = None;
    let mut needs_render = false;
    let mut should_advance_game_state = false;
    playable
        .animations
        .iter_mut()
        .for_each(|(animation_id, animation)| {
            let animation_index = animation_indices
                .entry(animation_id.to_string())
                .or_insert(0);
            let (step_needs_render, repaint_after_ms, new_animation_timestamp_ms) =
                EngineCore::process_animation_step(
                    animation_id,
                    animation,
                    &mut playable.images,
                    animation_index,
                    animation_timestamps_ms.get(animation_id).cloned(),
                    get_current_time_ms(),
                );
            needs_render |= step_needs_render;
            match (request_repaint, repaint_after_ms) {
                (Some(request_repaint_inner), Some(repaint_after_ms_inner)) => {
                    request_repaint =
                        Some(std::cmp::min(request_repaint_inner, repaint_after_ms_inner))
                }
                (None, Some(_)) => request_repaint = repaint_after_ms,
                _ => {}
            }
            match new_animation_timestamp_ms {
                Some(new_animation_timestamp_ms) => {
                    animation_timestamps_ms
                        .insert(animation_id.to_string(), new_animation_timestamp_ms);
                }
                None => {
                    animation_timestamps_ms.remove(animation_id);
                }
            }
            if animation.auto_advance && *animation_index >= animation.steps.len() {
                should_advance_game_state = true;
            }
        });

    // Queue repaint if needed
    if let Some(request_repaint) = request_repaint {
        if request_repaint == 0 {
            ui.ctx().request_repaint();
        } else {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(request_repaint));
        }
    }

    (needs_render, should_advance_game_state)
}
