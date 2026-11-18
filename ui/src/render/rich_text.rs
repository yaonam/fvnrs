use crate::constants::TEXT_DECORATION_STROKE_WIDTH;
use eframe::egui::{self, Color32, FontId, TextFormat, Ui, text::LayoutJob};
use state::{
    schema::{session_schema::TextScrollMode, text_schema::TextModifiers},
    settings::Settings,
    utils::get_current_time_ms,
};

enum TextType<'a> {
    Text(&'a str),
    Modifier(&'a str),
    /// Where the current text is scrolled to
    ScrollCursor,
}

pub(crate) fn rich_text_label(
    ui: &mut Ui,
    text_scroll_mode: &TextScrollMode,
    settings: &Settings,
    raw_text: &str,
    base_modifiers: &TextModifiers,
) -> bool {
    let (split_text, scroll_finished) = split_text(ui, raw_text, text_scroll_mode, settings);

    let mut job = LayoutJob::default();

    let base_format = into_text_format(base_modifiers);
    let mut current_format = base_format.clone();
    for text_seg in split_text {
        match text_seg {
            TextType::Text(text) => {
                job.append(text, 0.0, current_format.clone());
            }
            TextType::Modifier(modifier) => {
                apply_modifier(&mut current_format, &base_format, modifier);
            }
            TextType::ScrollCursor => {
                apply_scroll_cursor(&mut current_format);
            }
        }
    }

    // TODO: Manually paint to allow for outlines/drop shadows
    ui.label(job);

    scroll_finished
}

/// Splits the text on modifiers and scroll cursor.
/// Requests a repaint for the next scrolled character.
fn split_text<'a>(
    ui: &mut Ui,
    text: &'a str,
    text_scroll_mode: &TextScrollMode,
    settings: &Settings,
) -> (Vec<TextType<'a>>, bool) {
    let mut result = Vec::new();

    let text_start_ms = match text_scroll_mode {
        TextScrollMode::ScrollStart(start) => start,
        TextScrollMode::ScrollEnd(_) => &0,
    };

    // Calculate cursor
    let now = get_current_time_ms();
    let elapsed_ms = now - text_start_ms;
    let cursor_position_raw = elapsed_ms * settings.text_speed / 1000;
    let cursor_position = cursor_position_raw as usize;

    let mut scroll_finished = true;

    let mut request_repaint = false;

    let mut last_index = 0;
    // Count of chars, excluding modifiers
    let mut in_modifier = false;
    let mut accumulated_raw = 0;
    for (i, c) in text.char_indices() {
        if c == '{' {
            result.push(TextType::Text(&text[last_index..i]));
            last_index = i + c.len_utf8();
            in_modifier = true;
        } else if c == '}' {
            result.push(TextType::Modifier(&text[last_index..i]));
            last_index = i + c.len_utf8();
            in_modifier = false;
        } else if !in_modifier {
            if accumulated_raw as usize >= cursor_position {
                result.push(TextType::Text(&text[last_index..i]));
                result.push(TextType::ScrollCursor);
                last_index = i;
                request_repaint = true;
                scroll_finished = false;
                break;
            }
            if c == ',' {
                accumulated_raw += settings.comma_pause_ms * settings.text_speed / 1000;
            } else if c == '.' {
                accumulated_raw += settings.period_pause_ms * settings.text_speed / 1000;
            } else {
                accumulated_raw += 1;
            }
        }
    }
    result.push(TextType::Text(&text[last_index..]));

    if request_repaint {
        let ms_per_char = 1000 / settings.text_speed;
        let ms_to_wait = (ms_per_char - (elapsed_ms % ms_per_char)).max(0);
        if ms_to_wait == 0 {
            ui.ctx().request_repaint();
        } else {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(ms_to_wait));
        }
    }

    (result, scroll_finished)
}

// TODO: Support other fonts and bold
fn into_text_format(modifiers: &TextModifiers) -> TextFormat {
    let mut format = TextFormat::default();

    let text_color = match &modifiers.color {
        Some(color) => match Color32::from_hex(color) {
            Ok(color) => color,
            Err(_) => Color32::default(),
        },
        None => Color32::WHITE, // TODO: Move to settings or elsewhere
    };
    format.color = text_color;
    if let Some(size) = modifiers.size {
        format.font_id = FontId::proportional(size);
    }
    format.italics = modifiers.italic;
    if modifiers.underline {
        format.underline = egui::Stroke::new(TEXT_DECORATION_STROKE_WIDTH, text_color);
    }
    if modifiers.strikethrough {
        format.strikethrough = egui::Stroke::new(TEXT_DECORATION_STROKE_WIDTH, text_color);
    }

    format
}

fn apply_modifier(format: &mut TextFormat, base_format: &TextFormat, mut modifier: &str) {
    let is_close = modifier.starts_with('/');
    if is_close {
        modifier = &modifier[1..];
    }

    // Try assignments
    let (tag, arg) = modifier.split_once('=').unwrap_or((modifier, ""));

    match tag {
        "b" | "bold" => {
            // TODO
        }
        "i" | "italic" => {
            if is_close {
                format.italics = base_format.italics;
            } else {
                format.italics = true;
            }
        }
        "u" | "underline" => {
            if is_close {
                format.underline = base_format.underline;
            } else {
                format.underline = egui::Stroke::new(TEXT_DECORATION_STROKE_WIDTH, format.color);
            }
        }
        "s" | "strike" | "strikethrough" => {
            if is_close {
                format.strikethrough = base_format.strikethrough;
            } else {
                format.strikethrough =
                    egui::Stroke::new(TEXT_DECORATION_STROKE_WIDTH, format.color);
            }
        }
        "c" | "color" | "colour" => {
            if format.color == Color32::TRANSPARENT {
                return; // Don't override the transparent color
            }
            if is_close {
                format.color = base_format.color;
            } else {
                format.color = Color32::from_hex(arg).unwrap();
            }
        }
        "size" => {
            // TODO: Scale dynamically based on viewport width
            if is_close {
                format.font_id = base_format.font_id.clone();
            } else {
                format.font_id = FontId::proportional(arg.parse().unwrap());
            }
        }
        _ => {
            // Support other modifiers
        }
    }
}

fn apply_scroll_cursor(format: &mut TextFormat) {
    format.color = Color32::TRANSPARENT;
}
