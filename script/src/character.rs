pub trait Character {
    const NAME: &'static str;
    const COLOR: &'static str;
}

#[macro_export]
/// Defines a unit struct and a [`Character`] impl from a `name(...)` line.
///
/// ex. `character!(TestCharacter { name("Test Character"); });`
macro_rules! character {
    ($name:ident { name($display_name:expr); color($color:expr); }) => {
        #[allow(non_camel_case_types)]
        pub struct $name;

        impl $crate::character::Character for $name {
            const NAME: &'static str = $display_name;
            const COLOR: &'static str = $color;
        }
    };
}
