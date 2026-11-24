use eframe::egui::{Color32, FontFamily, FontId, Stroke, Vec2, vec2};

// ***********************************************************
// GLOBAL
// ***********************************************************

pub(crate) const INACTIVE_BG_FILL: Color32 =
    Color32::from_rgba_unmultiplied_const(255, 255, 255, 80);
pub(crate) const INACTIVE_WEAK_BG_FILL: Color32 = Color32::TRANSPARENT;
pub(crate) const INACTIVE_FG_STROKE: Stroke = Stroke {
    width: 1.0,
    color: Color32::WHITE,
};
pub(crate) const NONINTERACTIVE_FG_STROKE: Stroke = Stroke {
    width: 1.0,
    color: Color32::WHITE,
};
// Hover fill color for save lots, slider handle, and toggle
pub(crate) const HOVERED_BG_FILL: Color32 =
    Color32::from_rgba_unmultiplied_const(255, 255, 255, 150);
// Hover fill color for menu/overlay buttons
pub(crate) const HOVERED_WEAK_BG_FILL: Color32 =
    Color32::from_rgba_unmultiplied_const(255, 255, 255, 50);

pub(crate) const SKIP_DELAY_MS: u64 = 50;

// ***********************************************************
// MENUS
// ***********************************************************

pub(crate) const MENUS_TOP_OFFSET: f32 = 150.0;
pub(crate) const COLUMN_SPACER: f32 = 70.0;
pub(crate) const OVERLAY_H_DIVIDER: f32 = 0.25;
pub(crate) const EDGE_PADDING: f32 = 0.1; // 10% of screen size

pub(crate) const BUTTON_FONT_SIZE: f32 = 20.0;
pub(crate) const LARGE_BUTTON_FONT_SIZE: f32 = 45.0;
pub(crate) const BUTTON_PADDING: Vec2 = vec2(15.0, 12.0);

// ***********************************************************
// SAVES
// ***********************************************************

pub(crate) const SAVE_ITEM_TITLE_TEXT_SIZE: f32 = 45.0;
pub(crate) const SAVE_ITEM_DETAILS_TEXT_SIZE: f32 = 25.0;
pub(crate) const SAVE_ITEM_DETAILS_WIDTH: f32 = 700.0;
pub(crate) const SAVE_ITEM_DELETE_TEXT_SIZE: f32 = 40.0;
pub(crate) const SAVE_ITEM_DELETE_PADDING: Vec2 = vec2(20.0, 20.0);

pub(crate) const SAVE_SLOT_CORNER_RADIUS: f32 = 6.0;
pub(crate) const SAVE_SLOT_PADDING: f32 = 12.0;
pub(crate) const SAVE_SLOT_THUMB_SIZE: Vec2 = vec2(240.0, 135.0);
pub(crate) const SAVE_SLOT_HEIGHT: f32 = SAVE_SLOT_THUMB_SIZE.y + 2.0 * SAVE_SLOT_PADDING;

pub(crate) const SAVE_FADE_IN_TIME_MS: u64 = 30;
pub(crate) const SCREENSHOT_HEIGHT: u32 = 270;

// ***********************************************************
// DIALOGUE HUD
// ***********************************************************

pub(crate) const HUD_WIDTH_RATIO: f32 = 0.6; // Of viewport width
pub(crate) const HUD_HEIGHT_RATIO: f32 = 0.25; // Of viewport height
pub(crate) const HUD_MARGIN_RATIO: f32 = 0.025; // Of viewport width

pub(crate) const NVL_WIDTH_RATIO: f32 = 0.6; // Of viewport width
pub(crate) const NVL_SPACING: f32 = 45.0; // Between text

pub(crate) const CHARACTER_NAME_SIZE: f32 = 38.0;
pub(crate) const TEXT_SIZE: f32 = 33.0;
pub(crate) const TEXT_DECORATION_STROKE_WIDTH: f32 = 1.0;
pub(crate) const CONTROLS_INTERACT_SIZE: Vec2 = vec2(100.0, 50.0);
pub(crate) const CONTROLS_V_PADDING: f32 = 12.0;

pub(crate) const CONTROLS_ROW_HEIGHT: f32 = CONTROLS_INTERACT_SIZE.y + 2.0 * CONTROLS_V_PADDING;

// Grey
pub(crate) const HUD_BLUR_FILL: Color32 = Color32::from_rgba_unmultiplied_const(46, 57, 61, 100);
pub(crate) const HUD_BORDER_STROKE: Stroke = Stroke {
    width: 0.2,
    color: Color32::from_rgba_unmultiplied_const(255, 255, 255, 100),
};
// Dark grey
pub(crate) const NVL_FILL: Color32 = Color32::from_rgba_unmultiplied_const(14, 17, 18, 160);

pub(crate) const ACTIVATED_BUTTON_COLOR: Color32 = Color32::GRAY;
pub(crate) const NORMAL_BUTTON_COLOR: Color32 = Color32::WHITE;

pub(crate) const BASE_WAIT_TIME_MS: u64 = 700;

// ***********************************************************
// ROUTE SELECT WIDGET
// ***********************************************************

pub(crate) const ROUTE_SLOT_HEIGHT: f32 = 50.0;
pub(crate) const ROUTE_SLOT_WIDTH: f32 = 1000.0;
pub(crate) const ROUTE_SLOT_PADDING: f32 = 10.0;
pub(crate) const ROUTE_SLOT_CORNER_RADIUS: f32 = 5.0;

pub(crate) const ROUTE_SLOT_TEXT_SIZE: f32 = 30.0;

// ***********************************************************
// MULTIPLE CHOICE WIDGET
// ***********************************************************

pub(crate) const MC_SUBMIT_HEIGHT: f32 = 50.0;
pub(crate) const MC_SUBMIT_WIDTH: f32 = 100.0;

// ***********************************************************
// SETTINGS
// ***********************************************************

pub(crate) const SETTINGS_GRID_SPACING: Vec2 = vec2(50.0, 30.0);
pub(crate) const SETTINGS_INDENT: f32 = 50.0;

pub(crate) const SETTINGS_SECTION_TEXT_SIZE: f32 = 40.0;
pub(crate) const SETTINGS_TEXT_SIZE: f32 = 30.0;
pub(crate) const SETTINGS_SLIDER_TEXT_STYLE: FontId = FontId::new(25.0, FontFamily::Proportional);
pub(crate) const SETTINGS_SLIDER_TEXT_COLOR: Color32 =
    Color32::from_rgba_unmultiplied_const(200, 200, 200, 255);

pub(crate) const SETTINGS_SLIDER_WIDTH: f32 = 400.0;
// Toggle size
pub(crate) const SETTINGS_INTERACT_SIZE: Vec2 = vec2(40.0, 40.0);
pub(crate) const SETTINGS_ICON_WIDTH: f32 = 45.0;
pub(crate) const SETTINGS_BUTTON_PADDING: Vec2 = vec2(8.0, 6.0);
pub(crate) const SETTINGS_CORNER_RADIUS: f32 = SETTINGS_TEXT_SIZE / 2.0;

// Values
pub(crate) const MAX_TEXT_SPEED: u64 = 100;
pub(crate) const MIN_TEXT_WAIT: u64 = 10;
pub(crate) const MAX_TEXT_WAIT: u64 = 100;

pub(crate) const MAX_PUNCT_PAUSE: u64 = 1000;

pub(crate) const MAX_VOLUME: u8 = 100;

// ***********************************************************
// BLUR
// ***********************************************************

pub(crate) const BLUR_TRANSITION_MS: f32 = 100.0; // ms
// RGB values
pub(crate) const GREY_COLOR: u8 = 25;
// How much to divide the alpha by
pub(crate) const GREY_FACTOR: u8 = 2;

// ***********************************************************
// ABOUT
// ***********************************************************

pub(crate) const ABOUT_TEXT_SIZE: f32 = 25.0;
