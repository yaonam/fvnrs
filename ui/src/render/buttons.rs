use eframe::egui::{FontId, RichText, Ui};

use crate::constants::{BUTTON_FONT_SIZE, LARGE_BUTTON_FONT_SIZE};

pub(crate) fn button_text(text: &str) -> RichText {
    RichText::new(text).font(FontId::proportional(BUTTON_FONT_SIZE))
}

pub(crate) fn col_button_click<'a>(cols: &mut [Ui], idx: usize, text: &str) -> bool {
    cols[idx]
        .centered_and_justified(|ui| ui.button(button_text(text)))
        .inner
        .clicked()
}

pub(crate) fn menu_button_click<'a>(ui: &mut Ui, text: &str) -> bool {
    ui.button(RichText::new(text).font(FontId::proportional(LARGE_BUTTON_FONT_SIZE)))
        .clicked()
}
