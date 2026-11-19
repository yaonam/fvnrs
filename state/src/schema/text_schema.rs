use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct TextBoxSchema {
    pub character_name: String,
    pub character_name_color: Option<String>,
    pub text: String,
    pub text_color: Option<String>,

    /// For NVL mode. Keeps track of text within the scene.
    pub text_history: Option<Vec<String>>,

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
            text_history: None,
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

    pub fn enable_nvl_mode(&mut self) {
        if self.text_history.is_none() {
            self.text_history = Some(Vec::new());
        }
    }

    pub fn disable_nvl_mode(&mut self) {
        self.text_history = None;
    }

    pub fn apply(&mut self, say: &SaySchema) {
        self.character_name = say.character_name.to_string();
        self.character_name_color = Some(say.character_color.to_string());
        let old_text = std::mem::replace(&mut self.text, say.text.to_string());
        self.text_color = None;
        if let Some(history) = &mut self.text_history
            && !old_text.is_empty()
        {
            // NVL mode
            history.push(old_text);
        }
        // Retain position and visibility
        self.start_scroll = true;
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
