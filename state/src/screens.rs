use std::collections::HashMap;

use crate::{
    game_state::GameState,
    schema::{
        animation_schema::{AnimationMode, AnimationSchema},
        audio_schema::ChannelsSchema,
        image_schema::ImageSchema,
        playable_schema::PlayableSchema,
    },
    utils::get_current_time_ms,
};

#[derive(Clone, PartialEq, Debug)]
pub struct ScreensSchema {
    /// Splash screen with start time
    pub splash_screen: Option<(SplashType, u64)>,

    pub base_screen: BaseScreenSchema,
    /// Overlay screen, overlay open time, overlay change time
    pub overlay_screen: OverlayScreenSchema,
}

impl ScreensSchema {
    pub fn new(main_menu_playable: PlayableSchema) -> Self {
        Self {
            splash_screen: Some((SplashType::Startup, get_current_time_ms())),
            base_screen: BaseScreenSchema::MainMenu(main_menu_playable),
            overlay_screen: OverlayScreenSchema::default(),
        }
    }

    pub fn set_splash(&mut self, splash_type: SplashType) {
        self.splash_screen = Some((splash_type, get_current_time_ms()));
        self.close_menu();
    }

    pub fn set_game_state(&mut self, game_state: GameState) {
        self.base_screen = BaseScreenSchema::Game(game_state);
    }

    pub fn set_main_menu(&mut self, main_menu_playable: PlayableSchema) {
        self.base_screen = BaseScreenSchema::MainMenu(main_menu_playable);
    }

    /// Opens the menu overlay screen to the last overlay screen type.
    pub fn open_menu(&mut self) {
        let now = get_current_time_ms();
        self.overlay_screen.open_close_time = now;
        self.overlay_screen.update_time = now;
        self.overlay_screen.is_open = true;
    }

    /// Sets the overlay screen and updates the open time.
    /// If the overlay screen is already open, the open time is not updated.
    pub fn set_overlay_screen(&mut self, screen_type: OverlayScreenType) {
        let now = get_current_time_ms();
        if !self.overlay_screen.is_open {
            self.overlay_screen.open_close_time = now;
            self.overlay_screen.is_open = true;
        }
        self.overlay_screen.screen_type = screen_type;
        self.overlay_screen.update_time = now;
    }

    pub fn close_menu(&mut self) {
        let now = get_current_time_ms();
        self.overlay_screen.open_close_time = now;
        self.overlay_screen.update_time = now;
        self.overlay_screen.is_open = false;
    }

    /// Returns an iterator over all the current screen's images
    pub fn iter_images(&self) -> impl Iterator<Item = &Vec<ImageSchema>> {
        match &self.base_screen {
            BaseScreenSchema::MainMenu(playable) => playable.iter_images(),
            BaseScreenSchema::Game(state) => state.playable.iter_images(),
        }
    }

    /// Returns an iterator over all the current screen's animations
    pub fn iter_animations(&self) -> impl Iterator<Item = (&String, &AnimationSchema)> {
        match &self.base_screen {
            BaseScreenSchema::MainMenu(playable) => playable.iter_animations(),
            BaseScreenSchema::Game(state) => state.playable.iter_animations(),
        }
    }

    /// Returns an iterator over all the current screen's audio
    pub fn get_audio(&self) -> &ChannelsSchema {
        match &self.base_screen {
            BaseScreenSchema::MainMenu(playable) => &playable.audio,
            BaseScreenSchema::Game(state) => &state.playable.audio,
        }
    }

    pub fn get_game_state(&self) -> Option<&GameState> {
        match &self.base_screen {
            BaseScreenSchema::Game(game_state) => Some(game_state),
            _ => {
                log::warn!("No game state found");
                None
            }
        }
    }

    pub fn get_game_state_mut(&mut self) -> Option<&mut GameState> {
        match &mut self.base_screen {
            BaseScreenSchema::Game(game_state) => Some(game_state),
            _ => {
                log::warn!("No game state mut found");
                None
            }
        }
    }

    pub fn get_main_menu_state(&self) -> Option<&PlayableSchema> {
        match &self.base_screen {
            BaseScreenSchema::MainMenu(main_menu) => Some(main_menu),
            _ => {
                log::warn!("No main menu state found");
                None
            }
        }
    }

    pub fn get_main_menu_state_mut(&mut self) -> Option<&mut PlayableSchema> {
        match &mut self.base_screen {
            BaseScreenSchema::MainMenu(main_menu) => Some(main_menu),
            _ => {
                log::warn!("No main menu state mut found");
                None
            }
        }
    }

    pub fn is_blocking(&self, animation_indices: &HashMap<String, usize>) -> bool {
        // Blocking animations that are still playing
        self.iter_animations().any(|(key, animation)| {
            animation.mode == AnimationMode::Blocking
                && *animation_indices.get(key).unwrap_or(&0) < animation.steps.len()
        })
    }
}

#[derive(Clone, PartialEq, Debug, Copy)]
pub enum SplashType {
    Startup,
    Loading,
}

#[derive(Clone, PartialEq, Debug)]
pub enum BaseScreenSchema {
    MainMenu(PlayableSchema),
    Game(GameState),
}

#[derive(Clone, PartialEq, Debug, Copy, Default)]
/// Overlay screen that is displayed on top of the base screen
pub struct OverlayScreenSchema {
    pub screen_type: OverlayScreenType,
    pub open_close_time: u64,
    pub update_time: u64,
    pub is_open: bool,
}

#[derive(Clone, PartialEq, Debug, Copy, Default)]
/// Overlay screen that is displayed on top of the base screen
pub enum OverlayScreenType {
    #[default]
    Saves,
    Settings,
    About,
}
