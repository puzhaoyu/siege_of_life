use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::grid::Grid;
use crate::level::loader::LevelRegistry;
use crate::player::deploy::DragDeployState;
use crate::player::resources::DeploymentResources;
use crate::state::deployment::load_level_to_grid;
use crate::state::victory::{
    apply_trial_snapshot, DefeatKind, GameplayDefeatOverlay, GameplayVictoryOverlay, VictoryKind,
};
use crate::state::{
    AppState, CurrentLevelId, DeploymentZoneData, EvolutionConfig, SimulatorDeployConfig,
    SimulatorSnapshot, SimulatorState,
};

pub fn victory_overlay_ui(
    mut contexts: EguiContexts,
    mut victory_overlay: ResMut<GameplayVictoryOverlay>,
    state: Res<State<AppState>>,
    sim_state: Res<State<SimulatorState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut next_sim_state: ResMut<NextState<SimulatorState>>,
    mut current_level_id: ResMut<CurrentLevelId>,
    registry: Res<LevelRegistry>,
    mut grid: ResMut<Grid>,
    mut zone_data: ResMut<DeploymentZoneData>,
    mut deploy_res: ResMut<DeploymentResources>,
    mut evo_config: ResMut<EvolutionConfig>,
    mut drag_state: ResMut<DragDeployState>,
    snapshot: Res<SimulatorSnapshot>,
    deploy_config: Res<SimulatorDeployConfig>,
) {
    let Some(kind) = victory_overlay.kind.clone() else {
        return;
    };

    let ctx = contexts.ctx_mut();

    // 全屏半透明面板：吸收背景点击，中央窗口内的按钮可正常交互
    egui::CentralPanel::default()
        .frame(
            egui::Frame::none()
                .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 160)),
        )
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() * 0.28);

                egui::Frame::window(ui.style())
                    .inner_margin(egui::Margin::symmetric(32.0, 28.0))
                    .show(ui, |ui| {
                        ui.set_min_width(220.0);
                        ui.vertical_centered(|ui| {
                            ui.label(egui::RichText::new("闯关成功").size(32.0).strong());
                            ui.add_space(24.0);

                            match &kind {
                                VictoryKind::Level { has_next_level } => {
                                    if *has_next_level {
                                        if ui
                                            .add_sized(
                                                egui::vec2(200.0, 44.0),
                                                egui::Button::new(
                                                    egui::RichText::new("下一关").size(18.0),
                                                ),
                                            )
                                            .clicked()
                                        {
                                            go_next_level(
                                                &mut victory_overlay,
                                                &mut current_level_id,
                                                &registry,
                                                &mut grid,
                                                &mut zone_data,
                                                &mut deploy_res,
                                                &mut evo_config,
                                                &mut drag_state,
                                                &mut next_state,
                                            );
                                        }
                                        ui.add_space(8.0);
                                    }

                                    if ui
                                        .add_sized(
                                            egui::vec2(200.0, 44.0),
                                            egui::Button::new(
                                                egui::RichText::new("重新开始").size(18.0),
                                            ),
                                        )
                                        .clicked()
                                    {
                                        restart_current_level(
                                            &mut victory_overlay,
                                            &mut current_level_id,
                                            &registry,
                                            &mut grid,
                                            &mut zone_data,
                                            &mut deploy_res,
                                            &mut evo_config,
                                            &mut drag_state,
                                            &mut next_state,
                                        );
                                    }
                                    ui.add_space(8.0);

                                    if ui
                                        .add_sized(
                                            egui::vec2(200.0, 44.0),
                                            egui::Button::new(
                                                egui::RichText::new("返回").size(18.0),
                                            ),
                                        )
                                        .clicked()
                                    {
                                        victory_overlay.clear();
                                        *drag_state = DragDeployState::default();
                                        next_state.set(AppState::LevelSelect);
                                    }
                                }
                                VictoryKind::Trial => {
                                    if ui
                                        .add_sized(
                                            egui::vec2(200.0, 44.0),
                                            egui::Button::new(
                                                egui::RichText::new("重新开始").size(18.0),
                                            ),
                                        )
                                        .clicked()
                                    {
                                        restart_trial(
                                            &mut victory_overlay,
                                            &snapshot,
                                            &mut grid,
                                            &mut zone_data,
                                            &mut deploy_res,
                                            &deploy_config,
                                            &mut evo_config,
                                            &mut drag_state,
                                            &mut next_sim_state,
                                        );
                                    }
                                    ui.add_space(8.0);

                                    if ui
                                        .add_sized(
                                            egui::vec2(200.0, 44.0),
                                            egui::Button::new(
                                                egui::RichText::new("返回").size(18.0),
                                            ),
                                        )
                                        .clicked()
                                    {
                                        victory_overlay.clear();
                                        *drag_state = DragDeployState::default();
                                        evo_config.is_paused = true;
                                        evo_config.current_step = 0;
                                        evo_config.timer = 0.0;
                                        next_sim_state.set(SimulatorState::Editing);
                                    }
                                }
                            }
                        });
                    });
            });
        });

    // 胜利遮罩显示时保持演算暂停
    if *state.get() == AppState::Evolution
        || (*state.get() == AppState::Simulator && *sim_state.get() == SimulatorState::TrialPlay)
    {
        evo_config.is_paused = true;
    }
}

fn load_level_session(
    level_id: &str,
    data: &crate::level::data::LevelData,
    current_level_id: &mut CurrentLevelId,
    grid: &mut Grid,
    zone_data: &mut DeploymentZoneData,
    deploy_res: &mut DeploymentResources,
    evo_config: &mut EvolutionConfig,
    drag_state: &mut DragDeployState,
    next_state: &mut NextState<AppState>,
) {
    current_level_id.0 = Some(level_id.to_string());
    load_level_to_grid(grid, data);
    zone_data.zone = data.deployment_zone.clone();
    deploy_res.remaining_gliders = data.max_gliders;
    deploy_res.remaining_lwss = data.max_lwss;
    deploy_res.deployed_this_round = false;
    evo_config.steps_per_deployment = data.evolution_steps;
    evo_config.current_step = 0;
    evo_config.is_paused = true;
    evo_config.timer = 0.0;
    *drag_state = DragDeployState::default();
    next_state.set(AppState::Deployment);
}

fn restart_current_level(
    victory_overlay: &mut GameplayVictoryOverlay,
    current_level_id: &mut CurrentLevelId,
    registry: &LevelRegistry,
    grid: &mut Grid,
    zone_data: &mut DeploymentZoneData,
    deploy_res: &mut DeploymentResources,
    evo_config: &mut EvolutionConfig,
    drag_state: &mut DragDeployState,
    next_state: &mut NextState<AppState>,
) {
    let id = match &current_level_id.0 {
        Some(id) => id.clone(),
        None => return,
    };
    let data = match registry.get(&id) {
        Some(d) => d.clone(),
        None => return,
    };

    victory_overlay.clear();
    load_level_session(
        &id,
        &data,
        current_level_id,
        grid,
        zone_data,
        deploy_res,
        evo_config,
        drag_state,
        next_state,
    );
}

fn go_next_level(
    victory_overlay: &mut GameplayVictoryOverlay,
    current_level_id: &mut CurrentLevelId,
    registry: &LevelRegistry,
    grid: &mut Grid,
    zone_data: &mut DeploymentZoneData,
    deploy_res: &mut DeploymentResources,
    evo_config: &mut EvolutionConfig,
    drag_state: &mut DragDeployState,
    next_state: &mut NextState<AppState>,
) {
    let current_id = match &current_level_id.0 {
        Some(id) => id.clone(),
        None => return,
    };
    let idx = match registry.ordered_ids.iter().position(|i| i == &current_id) {
        Some(i) => i,
        None => return,
    };
    let next_id = match registry.ordered_ids.get(idx + 1) {
        Some(id) => id.clone(),
        None => return,
    };
    let data = match registry.get(&next_id) {
        Some(d) => d.clone(),
        None => return,
    };

    victory_overlay.clear();
    load_level_session(
        &next_id,
        &data,
        current_level_id,
        grid,
        zone_data,
        deploy_res,
        evo_config,
        drag_state,
        next_state,
    );
}

fn restart_trial(
    victory_overlay: &mut GameplayVictoryOverlay,
    snapshot: &SimulatorSnapshot,
    grid: &mut Grid,
    zone_data: &mut DeploymentZoneData,
    deploy_res: &mut DeploymentResources,
    deploy_config: &SimulatorDeployConfig,
    evo_config: &mut EvolutionConfig,
    drag_state: &mut DragDeployState,
    next_sim_state: &mut NextState<SimulatorState>,
) {
    if !apply_trial_snapshot(snapshot, grid, zone_data) {
        return;
    }

    victory_overlay.clear();
    deploy_res.remaining_gliders = deploy_config.max_gliders;
    deploy_res.remaining_lwss = deploy_config.max_lwss;
    deploy_res.deployed_this_round = false;
    evo_config.current_step = 0;
    evo_config.is_paused = true;
    evo_config.timer = 0.0;
    *drag_state = DragDeployState::default();
    next_sim_state.set(SimulatorState::DeploymentTest);
}

pub fn defeat_overlay_ui(
    mut contexts: EguiContexts,
    mut defeat_overlay: ResMut<GameplayDefeatOverlay>,
    state: Res<State<AppState>>,
    sim_state: Res<State<SimulatorState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut next_sim_state: ResMut<NextState<SimulatorState>>,
    mut current_level_id: ResMut<CurrentLevelId>,
    registry: Res<LevelRegistry>,
    mut grid: ResMut<Grid>,
    mut zone_data: ResMut<DeploymentZoneData>,
    mut deploy_res: ResMut<DeploymentResources>,
    mut evo_config: ResMut<EvolutionConfig>,
    mut drag_state: ResMut<DragDeployState>,
    snapshot: Res<SimulatorSnapshot>,
    deploy_config: Res<SimulatorDeployConfig>,
) {
    let Some(kind) = defeat_overlay.kind else {
        return;
    };
    let kind = kind;

    let ctx = contexts.ctx_mut();

    egui::CentralPanel::default()
        .frame(
            egui::Frame::none()
                .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 160)),
        )
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() * 0.28);

                egui::Frame::window(ui.style())
                    .inner_margin(egui::Margin::symmetric(32.0, 28.0))
                    .show(ui, |ui| {
                        ui.set_min_width(220.0);
                        ui.vertical_centered(|ui| {
                            ui.label(
                                egui::RichText::new("任务失败")
                                    .size(32.0)
                                    .strong()
                                    .color(egui::Color32::from_rgb(255, 120, 120)),
                            );
                            ui.add_space(12.0);
                            ui.label(
                                egui::RichText::new("部署资源已耗尽。")
                                    .size(16.0)
                                    .color(egui::Color32::from_rgb(220, 220, 220)),
                            );
                            ui.add_space(24.0);

                            match kind {
                                DefeatKind::Level => {
                                    if ui
                                        .add_sized(
                                            egui::vec2(200.0, 44.0),
                                            egui::Button::new(
                                                egui::RichText::new("重新开始").size(18.0),
                                            ),
                                        )
                                        .clicked()
                                    {
                                        let id = match &current_level_id.0 {
                                            Some(id) => id.clone(),
                                            None => return,
                                        };
                                        let data = match registry.get(&id) {
                                            Some(d) => d.clone(),
                                            None => return,
                                        };
                                        defeat_overlay.clear();
                                        load_level_session(
                                            &id,
                                            &data,
                                            &mut current_level_id,
                                            &mut grid,
                                            &mut zone_data,
                                            &mut deploy_res,
                                            &mut evo_config,
                                            &mut drag_state,
                                            &mut next_state,
                                        );
                                    }
                                    ui.add_space(8.0);

                                    if ui
                                        .add_sized(
                                            egui::vec2(200.0, 44.0),
                                            egui::Button::new(
                                                egui::RichText::new("返回").size(18.0),
                                            ),
                                        )
                                        .clicked()
                                    {
                                        defeat_overlay.clear();
                                        *drag_state = DragDeployState::default();
                                        next_state.set(AppState::LevelSelect);
                                    }
                                }
                                DefeatKind::Trial => {
                                    if ui
                                        .add_sized(
                                            egui::vec2(200.0, 44.0),
                                            egui::Button::new(
                                                egui::RichText::new("重新开始").size(18.0),
                                            ),
                                        )
                                        .clicked()
                                    {
                                        if !apply_trial_snapshot(&snapshot, &mut grid, &mut zone_data) {
                                            return;
                                        }
                                        defeat_overlay.clear();
                                        deploy_res.remaining_gliders = deploy_config.max_gliders;
                                        deploy_res.remaining_lwss = deploy_config.max_lwss;
                                        deploy_res.deployed_this_round = false;
                                        evo_config.current_step = 0;
                                        evo_config.is_paused = true;
                                        evo_config.timer = 0.0;
                                        *drag_state = DragDeployState::default();
                                        next_sim_state.set(SimulatorState::DeploymentTest);
                                    }
                                    ui.add_space(8.0);

                                    if ui
                                        .add_sized(
                                            egui::vec2(200.0, 44.0),
                                            egui::Button::new(
                                                egui::RichText::new("返回").size(18.0),
                                            ),
                                        )
                                        .clicked()
                                    {
                                        defeat_overlay.clear();
                                        *drag_state = DragDeployState::default();
                                        evo_config.is_paused = true;
                                        evo_config.current_step = 0;
                                        evo_config.timer = 0.0;
                                        next_sim_state.set(SimulatorState::Editing);
                                    }
                                }
                            }
                        });
                    });
            });
        });

    if *state.get() == AppState::Evolution
        || (*state.get() == AppState::Simulator && *sim_state.get() == SimulatorState::TrialPlay)
    {
        evo_config.is_paused = true;
    }
}