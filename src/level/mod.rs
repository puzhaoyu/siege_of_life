pub mod data;
pub mod loader;
pub mod progression;

use bevy::prelude::*;

use self::loader::load_builtin_levels;
use self::progression::load_progress;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        let registry = load_builtin_levels();
        let save_data = load_progress();
        app.insert_resource(registry);
        app.insert_resource(save_data);
    }
}
