use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::state::AppState;
use crate::ui::common::{big_button, centered_title};

pub fn main_menu_ui(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<AppState>>,
    state: Res<State<AppState>>,
) {
    if *state.get() != AppState::MainMenu {
        return;
    }

    let ctx = contexts.ctx_mut();

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.add_space(100.0);
        centered_title(ui, "围城之生");

        ui.add_space(20.0);
        ui.vertical_centered(|ui| {
            ui.label(
                egui::RichText::new("基于康威生命游戏的双阵营策略闯关游戏")
                    .size(14.0)
                    .weak(),
            );
        });

        ui.add_space(60.0);

        ui.vertical_centered(|ui| {
            if big_button(ui, "开始游戏").clicked() {
                next_state.set(AppState::LevelSelect);
            }
            ui.add_space(16.0);
            if big_button(ui, "自定义").clicked() {
                next_state.set(AppState::Simulator);
            }
        });
    });
}
