mod grid;
mod gol;
mod level;
mod state;
mod player;
mod ui;
mod input;
mod render;
mod save;
mod patterns;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;

use grid::GridPlugin;
use gol::GameOfLifePlugin;
use level::LevelPlugin;
use state::StatePlugin;
use player::PlayerPlugin;
use ui::UiPlugin;
use input::InputPlugin;
use render::RenderPlugin;
use save::SavePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "围城之生 - Siege of Life".to_string(),
                resolution: (1280.0, 800.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin)
        // 核心模块
        .add_plugins(GridPlugin)
        .add_plugins(GameOfLifePlugin)
        .add_plugins(LevelPlugin)
        // 状态机
        .add_plugins(StatePlugin)
        // 玩家系统
        .add_plugins(PlayerPlugin)
        // UI
        .add_plugins(UiPlugin)
        // 输入
        .add_plugins(InputPlugin)
        // 渲染
        .add_plugins(RenderPlugin)
        // 存档
        .add_plugins(SavePlugin)
        .run();
}
