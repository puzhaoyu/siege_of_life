use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::grid::Direction;
use crate::input::drag_drop::start_drag;
use crate::player::deploy::{DeployPhase, DeployUnitType, DragDeployState};
use crate::player::resources::DeploymentResources;
use crate::state::victory::GameplayVictoryOverlay;
use crate::state::{AppState, SimulatorState};

pub fn deployment_panel_ui(
    mut contexts: EguiContexts,
    state: Res<State<AppState>>,
    sim_state: Res<State<SimulatorState>>,
    mut drag_state: ResMut<DragDeployState>,
    deploy_res: Res<DeploymentResources>,
    mut next_state: ResMut<NextState<AppState>>,
    mut next_sim_state: ResMut<NextState<SimulatorState>>,
    overlay: Res<GameplayVictoryOverlay>,
) {
    let is_level_deploy = *state.get() == AppState::Deployment;
    let is_sim_deploy = *state.get() == AppState::Simulator
        && *sim_state.get() == SimulatorState::DeploymentTest;

    if !is_level_deploy && !is_sim_deploy {
        return;
    }

    if overlay.is_active() {
        return;
    }

    let ctx = contexts.ctx_mut();
    let unit_selected = drag_state.unit_type.is_some();

    egui::SidePanel::right("deployment_panel")
        .min_width(200.0)
        .resizable(false)
        .show(ctx, |ui| {
            ui.heading("部署阶段");
            ui.separator();
            ui.add_space(8.0);

            let (phase_text, status_color) = if let Some(unit) = drag_state.unit_type {
                let name = match unit {
                    DeployUnitType::Glider => "滑翔机",
                    DeployUnitType::LWSS => "轻型飞船",
                };
                match drag_state.phase {
                    DeployPhase::Rotating => {
                        let dir_str = match drag_state.rotation_dir {
                            Some(Direction::Up) => "上 ^",
                            Some(Direction::Down) => "下 v",
                            Some(Direction::Left) => "左 <",
                            Some(Direction::Right) => "右 >",
                            None => "右 >",
                        };
                        (format!("旋转中: {} 方向 [{}]", name, dir_str),
                         egui::Color32::from_rgb(255, 200, 50))
                    }
                    _ => {
                        (format!("已选: {} - 点击网格放置", name),
                         egui::Color32::from_rgb(200, 200, 100))
                    }
                }
            } else {
                ("待选择单位".to_string(),
                 egui::Color32::from_rgb(100, 200, 100))
            };

            ui.label(
                egui::RichText::new(&phase_text)
                    .size(13.0)
                    .color(status_color),
            );

            if drag_state.phase == DeployPhase::Rotating {
                ui.label(
                    egui::RichText::new("拖动鼠标旋转，点击确认")
                        .size(11.0)
                        .color(egui::Color32::from_rgb(200, 200, 200)),
                );
            }

            ui.add_space(8.0);
            ui.separator();

            ui.label(egui::RichText::new("可选单位:").size(14.0).strong());
            ui.add_space(4.0);

            let glider_btn_text = format!(
                "滑翔机 (Glider)  剩余: {}",
                deploy_res.remaining_gliders
            );
            let can_select = deploy_res.remaining_gliders > 0
                && !unit_selected
                && drag_state.phase == DeployPhase::Idle;

            let glider_response = ui.add_enabled_ui(can_select, |ui| {
                ui.add_sized(
                    egui::vec2(ui.available_width(), 32.0),
                    egui::Button::new(egui::RichText::new(&glider_btn_text).size(13.0)),
                )
            });
            if glider_response.inner.clicked() {
                start_drag(&mut drag_state, DeployUnitType::Glider);
            }

            ui.add_space(4.0);

            let lwss_btn_text = format!(
                "轻型飞船 (LWSS)  剩余: {}",
                deploy_res.remaining_lwss
            );
            let can_select = deploy_res.remaining_lwss > 0
                && !unit_selected
                && drag_state.phase == DeployPhase::Idle;

            let lwss_response = ui.add_enabled_ui(can_select, |ui| {
                ui.add_sized(
                    egui::vec2(ui.available_width(), 32.0),
                    egui::Button::new(egui::RichText::new(&lwss_btn_text).size(13.0)),
                )
            });
            if lwss_response.inner.clicked() {
                start_drag(&mut drag_state, DeployUnitType::LWSS);
            }

            ui.add_space(12.0);
            ui.separator();

            ui.label(egui::RichText::new("操作说明:").size(13.0).strong());
            ui.label(egui::RichText::new("1. 点击上方按钮选择单位").size(12.0));
            ui.label(egui::RichText::new("2. 图案跟随鼠标，点击落点").size(12.0));
            ui.label(egui::RichText::new("3. 拖动鼠标旋转方向(像旋钮)").size(12.0));
            ui.label(egui::RichText::new("4. 点击确认放置，自动开始演算").size(12.0));
            ui.label(egui::RichText::new("ESC 取消").size(12.0));

            ui.add_space(12.0);
            ui.separator();

            if unit_selected {
                if ui
                    .add_sized(
                        egui::vec2(ui.available_width(), 32.0),
                        egui::Button::new(
                            egui::RichText::new("取消选择 (ESC)").size(14.0),
                        ),
                    )
                    .clicked()
                {
                    drag_state.phase = DeployPhase::Idle;
                    drag_state.unit_type = None;
                    drag_state.rotation_dir = None;
                    drag_state.start_pos = None;
                    drag_state.current_pos = None;
                }
            } else {
                let back_text = if is_sim_deploy { "返回编辑" } else { "返回关卡选择" };
                if ui
                    .add_sized(
                        egui::vec2(ui.available_width(), 32.0),
                        egui::Button::new(
                            egui::RichText::new(back_text).size(14.0),
                        ),
                    )
                    .clicked()
                {
                    if is_sim_deploy {
                        next_sim_state.set(SimulatorState::Editing);
                    } else {
                        next_state.set(AppState::LevelSelect);
                    }
                }
            }

            ui.add_space(8.0);
            ui.separator();
            ui.label(egui::RichText::new("资源统计:").size(12.0).strong());
            ui.label(format!(
                "  滑翔机: {}  轻型飞船: {}",
                deploy_res.remaining_gliders, deploy_res.remaining_lwss
            ));
        });
}
