use serde::{Deserialize, Serialize};

use crate::schema::{
    playable_schema::PlayableSchema,
    schema::SceneSchema,
    text_schema::{SaySchema, TextBoxSchema},
};

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct GameState {
    /// None here means main menu.
    pub scene: Option<SceneSchema>,

    pub textbox: TextBoxSchema,

    pub playable: PlayableSchema,

    pub input_mode: GameStateInputMode,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            scene: None,
            textbox: TextBoxSchema::default(),
            playable: PlayableSchema::default(),
            input_mode: GameStateInputMode::Wait,
        }
    }
}

pub enum GameStateAction {
    Say(SaySchema),
    Function(fn(&mut GameState)),
}

pub trait SceneResolver: Send + Sync {
    fn resolve(&self, scene: usize) -> &'static [GameStateAction];

    fn reload(&mut self) {}
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug, Copy)]
pub enum GameStateInputMode {
    /// Wait for user input to continue
    Wait,
    /// Continue to next action without waiting for user input
    Continue,
}
