pub mod revival;
pub mod bomb;
pub mod engine;

use bevy::prelude::*;

pub struct GameOfLifePlugin;

impl Plugin for GameOfLifePlugin {
    fn build(&self, _app: &mut App) {
        // 演化引擎由状态系统直接调用，不需要独立 System
    }
}
