use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct TextBoxSchema {
    pub character_name: String,
    pub character_name_color: Option<String>,
    pub text: String,
    pub text_color: Option<String>,

    /// Float between [0,1] where origin is top left of screen. (x,y)
    pub position: Option<[f32; 2]>,
    pub visible: bool,

    /// Flip to true on new text
    pub start_scroll: bool,
}

impl Default for TextBoxSchema {
    fn default() -> Self {
        Self {
            character_name: String::new(),
            character_name_color: None,
            text: String::new(),
            text_color: None,
            position: None,
            visible: true,
            start_scroll: false,
        }
    }
}

impl TextBoxSchema {
    pub fn clear_start_scroll(&mut self) {
        self.start_scroll = false;
    }

    pub fn apply(&mut self, say: &SaySchema) {
        *self = Self {
            character_name: say.character_name.to_string(),
            character_name_color: Some(say.character_color.to_string()),
            text: say.text.to_string(),
            text_color: None,
            position: self.position, // Retain position
            visible: self.visible,   // Retain visibility
            start_scroll: true,      // Set to true
        };
    }
}

#[derive(Clone)]
pub struct SaySchema {
    pub character_name: &'static str,
    pub character_color: &'static str,
    pub text: &'static str,
}

// TODO: Refactor to LIFO stack?
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct TextModifiers {
    pub color: Option<String>,
    pub size: Option<f32>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
}

impl Default for TextModifiers {
    fn default() -> Self {
        Self {
            color: Some("#FFFFFF".to_string()),
            size: None,
            bold: false,
            italic: false,
            underline: false,
            strikethrough: false,
        }
    }
}
