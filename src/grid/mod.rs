pub mod types;
pub mod grid;

use bevy::prelude::*;

pub use types::*;
pub use grid::*;

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Grid::new(60, 40));
    }
}
