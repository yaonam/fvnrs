use eframe::egui::{self, FontId, Label, RichText, ScrollArea};

use crate::constants::ABOUT_TEXT_SIZE;

pub(crate) fn draw_about_view(ui: &mut egui::Ui, about_content: &str) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.add(
            Label::new(RichText::new(about_content).font(FontId::proportional(ABOUT_TEXT_SIZE)))
                .wrap(),
        )
    });
}
