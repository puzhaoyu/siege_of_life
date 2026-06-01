use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::level::data::SaveData;
use crate::level::loader::LevelRegistry;
use crate::state::{AppState, CurrentLevelId};
use crate::grid::Grid;

pub fn level_select_ui(
    _commands: Commands,
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<AppState>>,
    state: Res<State<AppState>>,
    mut current_level_id: ResMut<CurrentLevelId>,
    registry: Res<LevelRegistry>,
    save_data: Res<SaveData>,
    mut grid: ResMut<Grid>,
    mut evo_config: ResMut<crate::state::EvolutionConfig>,
    mut deploy_res: ResMut<crate::player::resources::DeploymentResources>,
) {
    if *state.get() != AppState::LevelSelect {
        return;
    }

    let ctx = contexts.ctx_mut();

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.add_space(30.0);
        ui.vertical_centered(|ui| {
            ui.heading("关卡选择");
        });

        ui.add_space(20.0);

        // 为底部「返回」预留高度，避免 ScrollArea 占满整屏把按钮挤出视口
        let footer_height = 70.0;
        let scroll_height = (ui.available_height() - footer_height).max(120.0);

        egui::ScrollArea::vertical()
            .max_height(scroll_height)
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    for (idx, id) in registry.ordered_ids.iter().enumerate() {
                        let level_data = match registry.get(id) {
                            Some(d) => d,
                            None => continue,
                        };
                        let unlocked = save_data.unlocked_levels.contains(id);
                        let completed = save_data.completed_levels.contains(id);

                        let status_text = if completed {
                            "✓"
                        } else if unlocked {
                            ""
                        } else {
                            "🔒"
                        };

                        ui.add_space(4.0);
                        let label_text = if unlocked {
                            format!(
                                "{}  {} - {}      滑翔机:{} LWSS:{}",
                                status_text,
                                idx + 1,
                                level_data.name,
                                level_data.max_gliders,
                                level_data.max_lwss
                            )
                        } else {
                            format!("{}  第{}关 - ???", status_text, idx + 1)
                        };

                        let button = egui::Button::new(egui::RichText::new(&label_text).size(16.0))
                            .min_size(egui::vec2(350.0, 40.0));

                        let response = if !unlocked {
                            ui.add_enabled(false, button)
                        } else {
                            ui.add(button)
                        };

                        if response.clicked() {
                            current_level_id.0 = Some(id.clone());
                            crate::state::deployment::load_level_to_grid(&mut grid, level_data);
                            evo_config.steps_per_deployment = level_data.evolution_steps;
                            evo_config.current_step = 0;
                            evo_config.is_paused = true;
                            evo_config.timer = 0.0;
                            deploy_res.remaining_gliders = level_data.max_gliders;
                            deploy_res.remaining_lwss = level_data.max_lwss;
                            deploy_res.deployed_this_round = false;
                            next_state.set(AppState::Deployment);
                        }
                    }
                });
            });

        ui.add_space(12.0);

        ui.vertical_centered(|ui| {
            if ui
                .add_sized(
                    egui::vec2(150.0, 40.0),
                    egui::Button::new(egui::RichText::new("返回").size(18.0)),
                )
                .clicked()
            {
                next_state.set(AppState::MainMenu);
            }
        });
    });
}
