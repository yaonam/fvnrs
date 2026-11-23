use std::time::Duration;

use eframe::egui::{self, InputState, MouseWheelUnit, Response};
use script::scene::jump;
use state::{
    game_state::GameStateInputMode,
    schema::session_schema::{TextScrollMode, UserInputOverride},
    utils::get_current_time_ms,
};

use crate::{
    constants::{BASE_WAIT_TIME_MS, MAX_TEXT_SPEED, SKIP_DELAY_MS},
    ui::FvnrsApp,
};

pub(crate) fn handle_global_inputs(app: &mut FvnrsApp, ctx: &egui::Context) {
    // Lost focus
    let focused = ctx.input(|i| {
        i.events.iter().rev().find_map(|event| match event {
            egui::Event::WindowFocused(focused) => Some(*focused),
            _ => None,
        })
    });
    if let Some(focused) = focused {
        if focused || !app.engine_core.read().persistent.settings.play_in_bg {
            app.audio.set_paused(!focused);
        }
    }

    // Fullscreen toggle
    if ctx.input(|i| i.key_pressed(egui::Key::F11)) {
        let mut core_write = app.engine_core.write();
        let fullscreen = core_write.persistent.settings.fullscreen;

        // Update settings
        core_write.persistent.settings.fullscreen = !fullscreen;
        // Send persistent update to persist worker
        let _ = core_write.persist_worker_tx.try_send(());
    }
}

pub(crate) fn handle_game_screen_interaction(
    app: &mut FvnrsApp,
    ui: &egui::Ui,
    response: Response,
) -> bool {
    let ctx = ui.ctx();

    // Hide hud
    if ctx.input(|i| i.key_pressed(egui::Key::H)) {
        let mut core_write = app.engine_core.write();

        if core_write.session.user_input_override != Some(UserInputOverride::Hide) {
            core_write.session.user_input_override = Some(UserInputOverride::Hide);
        } else {
            core_write.session.user_input_override = None;
        }
        return true;
    }

    // Toggle skip
    if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Tab)) {
        // Prevent egui from focusing/selecting UI elements
        ctx.memory_mut(|m| {
            m.move_focus(egui::FocusDirection::None);
            if let Some(id) = m.focused() {
                m.surrender_focus(id);
            }
        });

        let mut core_write = app.engine_core.write();

        if core_write.session.user_input_override != Some(UserInputOverride::Skip) {
            core_write.session.user_input_override = Some(UserInputOverride::Skip);
        } else {
            core_write.session.user_input_override = None;
        }
        return true;
    }

    // Handle state inputs
    if response.clicked() || ctx.input(|i| i.key_pressed(egui::Key::Space)) {
        let mut core_write = app.engine_core.write();
        if core_write.session.user_input_override.is_some() {
            // Remove user input override
            core_write.session.user_input_override = None;
        } else if matches!(
            core_write.session.text_scroll_mode,
            TextScrollMode::ScrollStart(_)
        ) {
            // End the scrolling
            core_write.session.end_text_scroll();
        } else {
            drop(core_write);
            return handle_next(app);
        }
    } else if response.secondary_clicked() {
        // Open menu
        let mut core_write = app.engine_core.write();
        if core_write.session.user_input_override.is_some() {
            // Remove user input override
            core_write.session.user_input_override = None;
        }
        core_write.screens.open_menu();
    } else if ctx.input(|i| is_mouse_wheel_down_event(i)) {
        let mut core_write = app.engine_core.write();
        // Next state if not already seen
        core_write.next_if_already_seen();
        // End the scrolling
        core_write.session.end_text_scroll();
        // Render screen
        if core_write.attr_changed(|s| &s.playable.images) {
            return true;
        }
    } else if ctx.input(|i| is_mouse_wheel_up_event(i)) {
        let mut core_write = app.engine_core.write();
        core_write.back();
        // Render screen
        if core_write.attr_changed_rev(|s| &s.playable.images) {
            return true;
        }
    }
    false
}

pub(crate) fn handle_user_overrides(app: &mut FvnrsApp, ctx: &egui::Context) -> bool {
    let mut render = false;
    let mut repaint = false;
    let mut repaint_after = None;
    // Check UserInputMode
    let mut core_write = app.engine_core.write();
    // Override override if CTRL is pressed
    let input_mode = if ctx.input(|i| i.modifiers.ctrl) {
        core_write.session.user_input_override = None;
        Some(UserInputOverride::Skip)
    } else {
        core_write.session.user_input_override
    };
    match input_mode {
        Some(UserInputOverride::Auto) => {
            // Wait for the scrolling to finish
            if let TextScrollMode::ScrollEnd(end_ms) = core_write.session.text_scroll_mode {
                let Some(game_state) = core_write.screens.get_game_state() else {
                    log::warn!("Couldn't get game state for Auto");
                    return false;
                };

                // Check the time
                let wait_time_per_char_ms = core_write.persistent.settings.text_wait;
                let text_len = game_state.textbox.text.len();
                let now = get_current_time_ms();
                let wait_end = end_ms + text_len as u64 * wait_time_per_char_ms + BASE_WAIT_TIME_MS;

                if now >= wait_end {
                    drop(core_write);
                    render = handle_next(app);
                    repaint = true;
                } else {
                    repaint_after = Some(Duration::from_millis(wait_end - now));
                }
            }
        }
        Some(UserInputOverride::Skip) => {
            drop(core_write);
            {
                let mut core_write = app.engine_core.write();
                // TODO: Have a global max scene schema?
                match core_write.persistent.settings.skip_unseen {
                    true => core_write.next(),
                    false => core_write.next_if_already_seen_global(),
                };
                render = core_write.attr_changed(|s| &s.playable.images);
            }
            // Add a small delay
            repaint_after = Some(Duration::from_millis(SKIP_DELAY_MS));
        }
        _ => {}
    }

    // Repaint if needed
    if repaint {
        ctx.request_repaint();
    } else if let Some(duration) = repaint_after {
        ctx.request_repaint_after(duration);
    }
    render
}

/// Advances to the next state. Returns whether the images changed.
pub(crate) fn handle_next(app: &mut FvnrsApp) -> bool {
    let mut core_write = app.engine_core.write();
    if let Some(game_state) = core_write.screens.get_game_state() {
        if game_state.input_mode == GameStateInputMode::AdvanceBlocked {
            return false;
        }
    }
    core_write.next();
    let Some(game_state) = core_write.screens.get_game_state() else {
        return false;
    };
    if game_state.textbox.start_scroll {
        if core_write.persistent.settings.text_speed == MAX_TEXT_SPEED {
            core_write.session.text_scroll_mode = TextScrollMode::ScrollEnd(get_current_time_ms());
        } else {
            core_write.session.start_text_scroll();
        }
    }
    core_write.attr_changed(|s| &s.playable.images)
}

/// Advances the scene. Returns whether the images changed.
pub(crate) fn handle_jump(app: &mut FvnrsApp, scene_id: usize) -> bool {
    {
        let mut core_write = app.engine_core.write();
        let Some(game_state) = core_write.screens.get_game_state_mut() else {
            return false;
        };
        game_state.widget = None;
        jump(game_state, scene_id); // Clears the AdvanceBlocked input mode
        core_write.refresh_cache();
    }
    // Handle the rest
    handle_next(app)
}

fn is_mouse_wheel_down_event(i: &InputState) -> bool {
    i.events.iter().any(|e| match e {
        egui::Event::MouseWheel { unit, delta, .. } => {
            if unit == &MouseWheelUnit::Line && delta.y < 0.0 {
                true
            } else {
                false
            }
        }
        _ => false,
    })
}

fn is_mouse_wheel_up_event(i: &InputState) -> bool {
    i.events.iter().any(|e| match e {
        egui::Event::MouseWheel { unit, delta, .. } => {
            if unit == &MouseWheelUnit::Line && delta.y > 0.0 {
                true
            } else {
                false
            }
        }
        _ => false,
    })
}
