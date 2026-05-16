use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::player::resources::DeploymentResources;
use crate::state::{AppState, EvolutionConfig};

pub fn hud_ui(
    mut contexts: EguiContexts,
    state: Res<State<AppState>>,
    evo_config: Res<EvolutionConfig>,
    deploy_res: Res<DeploymentResources>,
) {
    let ctx = contexts.ctx_mut();

    egui::TopBottomPanel::top("hud_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            // 演化状态
            let status_text = if evo_config.is_paused {
                "已暂停"
            } else {
                "运行中"
            };
            ui.label(
                egui::RichText::new(format!("状态: {}", status_text))
                    .size(14.0),
            );

            ui.separator();

            // 演化步数
            if *state.get() == AppState::Evolution {
                ui.label(
                    egui::RichText::new(format!(
                        "步数: {}/{}",
                        evo_config.current_step, evo_config.steps_per_deployment
                    ))
                    .size(14.0),
                );
            }

            ui.separator();

            // 部署资源
            if *state.get() == AppState::Deployment || *state.get() == AppState::Evolution {
                ui.label(
                    egui::RichText::new(format!(
                        "滑翔机: {}  LWSS: {}",
                        deploy_res.remaining_gliders, deploy_res.remaining_lwss
                    ))
                    .size(14.0),
                );
            }

            ui.separator();

            // 速度
            ui.label(
                egui::RichText::new(format!("速度: {:.0}ms/步", evo_config.speed_ms))
                    .size(14.0),
            );
        });
    });
}
