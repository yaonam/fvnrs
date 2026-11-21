use eframe::egui::{self, Frame, Ui};

use crate::ui::FvnrsApp;

pub fn draw_route_select(app: &mut FvnrsApp, ui: &mut Ui) -> bool {
    let viewport = ui.max_rect();
    let size = egui::vec2(viewport.width() * NVL_WIDTH_RATIO, viewport.height());
    let pos = egui::pos2(viewport.center().x - size.x / 2.0, viewport.top());
    let rect = egui::Rect::from_min_size(pos, size);

    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        let hud_frame = 
            // Overlay hud frame
            Frame::default()
                .inner_margin(scaled_margin)
                .shadow(epaint::Shadow {
                    color: HUD_BLUR_FILL,
                    offset: [0, 4],
                    blur: 20,
                    spread: 17,
                });

        hud_frame.show(ui, |ui| {});
    });

    false
}
