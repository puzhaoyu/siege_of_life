pub mod drag_drop;
pub mod rotation;
pub mod drawing;

use bevy::prelude::*;

use self::drawing::drawing_system;
use self::drag_drop::drag_drop_system;
use self::rotation::rotation_system;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                drawing_system,
                drag_drop_system,
                rotation_system,
            ),
        );
    }
}
