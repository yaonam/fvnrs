use state::{
    game_state::{GameState, GameStateInputMode},
    schema::schema::SceneSchema,
};

#[macro_export]
macro_rules! scene {
    // Entry point
    ($name:ident { $($input:tt)* }) => {
        $crate::scene! { @parse $name [] $($input)* }
    };

    // Base case
    (@parse $name:ident [$($actions:expr,)*]) => {
        // Statically compiled action table for this scene.
        pub static $name: &[$crate::state::game_state::GameStateAction] = &[$($actions),*];
    };

    // Speaker line without semicolon: `Name "literal"`
    (@parse $name:ident [$($actions:expr,)*] $sp:ident $msg:literal $($rest:tt)*) => {
        $crate::scene!(
            @parse $name
            [
                $($actions,)*
                $crate::state::game_state::GameStateAction::Say($crate::state::schema::text_schema::SaySchema {
                    character_name: <$sp as $crate::character::Character>::NAME,
                    character_color: <$sp as $crate::character::Character>::COLOR,
                    text: $msg,
                }),
            ]
            $($rest)*
        );
    };

    // Narrator line without semicolon: `"literal"`
    (@parse $name:ident [$($actions:expr,)*] $msg:literal $($rest:tt)*) => {
        $crate::scene!(
            @parse $name
            [
                $($actions,)*
                $crate::state::game_state::GameStateAction::Say($crate::state::schema::text_schema::SaySchema {
                    character_name: "",
                    character_color: "#FFFFFF",
                    text: $msg,
                }),
            ]
            $($rest)*
        );
    };

    // Function where we inject the mutable game state reference. Supports chaining methods.
    (@parse $name:ident [$($actions:expr,)*] $($func:ident)::* ( $($args:expr),* $(,)? ) $( . $method:ident ( $($m_args:expr),* $(,)? ) )* ; $($rest:tt)*) => {
        $crate::scene!(@parse $name [
            $($actions,)*
            $crate::state::game_state::GameStateAction::Function(
                |game_state: &mut $crate::state::game_state::GameState| {
                    $($func)::*(game_state $(, $args)*) $( . $method ( $($m_args),* ) )*;
                },
            ),
        ] $($rest)*);
    };

    // Arbitrary function that takes in the mutable game state reference
    (@parse $name:ident [$($actions:expr,)*] $action:expr ; $($rest:tt)*) => {
        $crate::scene!(@parse $name [
            $($actions,)*
            $crate::state::game_state::GameStateAction::Function(
                |game_state: &mut $crate::state::game_state::GameState| {
                    ($action)(game_state)
                },
            ),
        ] $($rest)*);
    };
}

pub fn jump(game_state: &mut GameState, next_scene_id: impl Into<usize>) {
    game_state.scene = Some(SceneSchema::new(next_scene_id.into()));
    game_state.input_mode = GameStateInputMode::Continue;
}

pub fn main_menu(game_state: &mut GameState) {
    *game_state = GameState::default();
}
