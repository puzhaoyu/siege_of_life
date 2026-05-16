use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::grid::{CellType, Faction, Grid};
use crate::gol::engine;
use crate::level::data::LevelData;
use crate::patterns;
use crate::player::deploy::DragDeployState;
use crate::player::resources::DeploymentResources;
use crate::state::{
    AppState, DeploymentZoneData, DialogMessage, EvolutionConfig, SelectedElement, SelectedPattern,
    SimulatorDeployConfig, SimulatorSnapshot, SimulatorState, ZoneBrushConfig,
};
use crate::ui::common::show_confirm_dialog;

/// 模拟器侧面板
pub fn simulator_panel_ui(
    mut contexts: EguiContexts,
    state: Res<State<AppState>>,
    sim_state: Res<State<SimulatorState>>,
    mut grid: ResMut<Grid>,
    mut evo_config: ResMut<EvolutionConfig>,
    mut selected_element: ResMut<SelectedElement>,
    mut selected_pattern: ResMut<SelectedPattern>,
    mut dialog: ResMut<DialogMessage>,
    mut next_state: ResMut<NextState<AppState>>,
    mut next_sim_state: ResMut<NextState<SimulatorState>>,
    mut deploy_config: ResMut<SimulatorDeployConfig>,
    mut deploy_res: ResMut<DeploymentResources>,
    mut snapshot: ResMut<SimulatorSnapshot>,
    mut drag_state: ResMut<DragDeployState>,
    mut zone_data: ResMut<DeploymentZoneData>,
    mut zone_brush: ResMut<ZoneBrushConfig>,
) {
    if *state.get() != AppState::Simulator {
        return;
    }

    // 部署测试模式由 deployment_panel_ui 处理，不显示本面板
    if *sim_state.get() == SimulatorState::DeploymentTest {
        return;
    }

    let ctx = contexts.ctx_mut();

    // 弹出对话框
    let show_msg = dialog.0.clone();
    if let Some(ref msg) = show_msg {
        let mut closed = false;
        crate::ui::common::show_dialog(ctx, msg, || {
            closed = true;
        });
        if closed {
            dialog.0 = None;
        }
    }

    // 右侧面板
    egui::SidePanel::right("simulator_panel")
        .min_width(220.0)
        .show(ctx, |ui| {
            ui.heading("模拟器");

            ui.add_space(8.0);

            // 当前模式
            let mode_label = match *sim_state.get() {
                SimulatorState::Editing => "编辑模式",
                SimulatorState::TrialPlay => "试玩模式 (演化中)",
                SimulatorState::DeploymentTest => "试玩模式 (部署中)",
            };
            ui.label(
                egui::RichText::new(format!("当前: {}", mode_label))
                    .size(13.0)
                    .strong(),
            );

            ui.add_space(8.0);
            ui.separator();

            // ---- 元素选择 (仅编辑模式) ----
            if *sim_state.get() == SimulatorState::Editing {
                ui.label(egui::RichText::new("放置元素").strong());

                egui::ComboBox::from_label("类型")
                    .selected_text(cell_type_name(&selected_element.0))
                    .show_ui(ui, |ui| {
                        for ct in &[
                            CellType::Normal(Faction::Red),
                            CellType::Normal(Faction::Blue),
                            CellType::Wall,
                            CellType::Bomb,
                            CellType::HighValue,
                        ] {
                            if ui
                                .selectable_value(
                                    &mut selected_element.0,
                                    *ct,
                                    cell_type_name(ct),
                                )
                                .clicked()
                            {
                                selected_pattern.0 = None;
                            }
                        }
                    });

                // 预设图案
                ui.add_space(8.0);
                ui.label(egui::RichText::new("预设图案").strong());
                egui::ComboBox::from_label("图案")
                    .selected_text(selected_pattern.0.as_deref().unwrap_or("无"))
                    .show_ui(ui, |ui| {
                        if ui
                            .selectable_value(&mut selected_pattern.0, None, "无")
                            .clicked()
                        {}
                        scrollable_pattern_list(
                            ui,
                            &mut selected_pattern.0,
                            patterns::still_life_names(),
                        );
                        ui.separator();
                        scrollable_pattern_list(
                            ui,
                            &mut selected_pattern.0,
                            patterns::oscillator_names(),
                        );
                        ui.separator();
                        scrollable_pattern_list(
                            ui,
                            &mut selected_pattern.0,
                            patterns::deployable_names(),
                        );
                    });

                ui.add_space(8.0);
                ui.separator();

                // ---- 部署设置 ----
                ui.label(egui::RichText::new("试玩部署设置").strong());
                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    ui.label("滑翔机:");
                    let mut gliders = deploy_config.max_gliders as i32;
                    if ui
                        .add(egui::DragValue::new(&mut gliders).range(0..=10).speed(0.1))
                        .changed()
                    {
                        deploy_config.max_gliders = gliders.max(0) as u32;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("轻型飞船:");
                    let mut lwss = deploy_config.max_lwss as i32;
                    if ui
                        .add(egui::DragValue::new(&mut lwss).range(0..=5).speed(0.1))
                        .changed()
                    {
                        deploy_config.max_lwss = lwss.max(0) as u32;
                    }
                });

                ui.add_space(8.0);
                ui.separator();

                // ---- 部署区域编辑 ----
                ui.label(egui::RichText::new("部署区域").strong());
                ui.add_space(4.0);

                let brush_label = if zone_brush.active {
                    "画笔: 开启"
                } else {
                    "画笔: 关闭"
                };
                if ui.button(brush_label).clicked() {
                    zone_brush.active = !zone_brush.active;
                }

                ui.horizontal(|ui| {
                    ui.label("画笔大小:");
                    let mut size = zone_brush.size as i32;
                    if ui.add(egui::Slider::new(&mut size, 1..=5)).changed() {
                        zone_brush.size = size as u32;
                    }
                });

                ui.label(format!("区域格数: {}", zone_data.zone.len()));

                if ui.button("清除区域").clicked() {
                    zone_data.zone.clear();
                }

                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("左键绘制 / 右键擦除")
                        .weak()
                        .size(11.0),
                );

                ui.add_space(8.0);
                ui.separator();

                // ---- 试玩按钮 ----
                if ui
                    .add_sized(
                        egui::vec2(180.0, 36.0),
                        egui::Button::new(
                            egui::RichText::new("进入试玩")
                                .size(15.0)
                                .color(egui::Color32::from_rgb(100, 255, 100)),
                        ),
                    )
                    .clicked()
                {
                    // 保存当前网格快照和区域数据
                    snapshot.cells = Some(grid.cells.clone());
                    snapshot.zone = Some(zone_data.zone.clone());
                    // 关闭区域画笔
                    zone_brush.active = false;
                    // 初始化部署资源
                    deploy_res.remaining_gliders = deploy_config.max_gliders;
                    deploy_res.remaining_lwss = deploy_config.max_lwss;
                    deploy_res.deployed_this_round = false;
                    // 重置拖拽状态
                    *drag_state = DragDeployState::default();
                    // 进入部署测试模式
                    next_sim_state.set(SimulatorState::DeploymentTest);
                }

                ui.add_space(8.0);
                ui.separator();
            }

            // ---- 演化控制 (试玩模式) ----
            if *sim_state.get() == SimulatorState::TrialPlay {
                ui.label(egui::RichText::new("演化控制").strong());

                ui.horizontal(|ui| {
                    if ui
                        .button(if evo_config.is_paused {
                            "继续"
                        } else {
                            "暂停"
                        })
                        .clicked()
                    {
                        evo_config.is_paused = !evo_config.is_paused;
                    }
                    if ui.button("单步").clicked() {
                        let (_changed, _bomb_result) = engine::evolution_step(&mut grid);
                        engine::check_high_value_destruction(&mut grid);
                        evo_config.current_step += 1;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("速度(ms):");
                    let mut speed = evo_config.speed_ms;
                    if ui
                        .add(egui::Slider::new(&mut speed, 10.0..=1000.0).text("ms"))
                        .changed()
                    {
                        evo_config.speed_ms = speed;
                    }
                });

                ui.label(format!("已演化: {} 步", evo_config.current_step));

                ui.add_space(8.0);
                ui.separator();
            }

            // ---- 返回编辑按钮 (部署测试 / 试玩模式) ----
            if *sim_state.get() == SimulatorState::DeploymentTest
                || *sim_state.get() == SimulatorState::TrialPlay
            {
                ui.add_space(4.0);
                if ui
                    .add_sized(
                        egui::vec2(180.0, 36.0),
                        egui::Button::new(egui::RichText::new("返回编辑").size(15.0)),
                    )
                    .clicked()
                {
                    // 恢复网格快照
                    if let Some(ref saved_cells) = snapshot.cells {
                        grid.cells = saved_cells.clone();
                    }
                    // 恢复区域数据
                    if let Some(ref saved_zone) = snapshot.zone {
                        zone_data.zone = saved_zone.clone();
                    }
                    snapshot.cells = None;
                    snapshot.zone = None;
                    // 重置演化状态
                    evo_config.is_paused = true;
                    evo_config.current_step = 0;
                    evo_config.timer = 0.0;
                    // 重置拖拽状态
                    *drag_state = DragDeployState::default();
                    // 返回编辑模式
                    next_sim_state.set(SimulatorState::Editing);
                }

                ui.add_space(8.0);
                ui.separator();
            }

            // ---- 辅助功能 (仅编辑模式) ----
            if *sim_state.get() == SimulatorState::Editing {
                ui.label(egui::RichText::new("辅助").strong());

                if ui.button("清空全部").clicked() {
                    show_confirm_dialog(
                        ctx,
                        "确认清空",
                        "确定要清除网格上所有内容吗？",
                        || {
                            grid.clear();
                        },
                        || {},
                    );
                }

                ui.add_space(8.0);
                ui.separator();

                // ---- 导出/导入 ----
                ui.label(egui::RichText::new("关卡管理").strong());

                if ui.button("导出 JSON").clicked() {
                    let width = grid.width;
                    let height = grid.height;
                    let level_data = LevelData {
                        id: "custom_level".to_string(),
                        name: "自定义关卡".to_string(),
                        version: 1,
                        width,
                        height,
                        initial_cells: grid.cells.clone(),
                        deployment_zone: zone_data.zone.clone(),
                        max_gliders: deploy_config.max_gliders,
                        max_lwss: deploy_config.max_lwss,
                        evolution_steps: 200,
                    };
                    if let Ok(json) = serde_json::to_string_pretty(&level_data) {
                        let path = std::env::current_dir()
                            .unwrap_or_default()
                            .join("custom_level.json");
                        let _ = std::fs::write(&path, &json);
                        dialog.0 = Some(format!("已导出到: {}", path.display()));
                    }
                }

                if ui.button("导入 JSON").clicked() {
                    let path = std::env::current_dir()
                        .unwrap_or_default()
                        .join("custom_level.json");
                    if path.exists() {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if let Ok(data) = serde_json::from_str::<LevelData>(&content) {
                                crate::state::deployment::load_level_to_grid(&mut grid, &data);
                                deploy_config.max_gliders = data.max_gliders;
                                deploy_config.max_lwss = data.max_lwss;
                                zone_data.zone = data.deployment_zone.clone();
                                dialog.0 = Some("关卡已导入".to_string());
                            }
                        }
                    } else {
                        dialog.0 = Some("未找到 custom_level.json".to_string());
                    }
                }

                ui.add_space(8.0);
                ui.separator();
            }

            // ---- 返回主菜单 ----
            if ui
                .add_sized(
                    egui::vec2(180.0, 36.0),
                    egui::Button::new(egui::RichText::new("返回主菜单").size(16.0)),
                )
                .clicked()
            {
                evo_config.is_paused = true;
                evo_config.current_step = 0;
                evo_config.timer = 0.0;
                snapshot.cells = None;
                snapshot.zone = None;
                *drag_state = DragDeployState::default();
                zone_brush.active = false;
                next_sim_state.set(SimulatorState::Editing);
                next_state.set(AppState::MainMenu);
            }
        });
}

fn cell_type_name(ct: &CellType) -> &'static str {
    match ct {
        CellType::Empty => "空",
        CellType::Normal(Faction::Red) => "红方细胞",
        CellType::Normal(Faction::Blue) => "蓝方细胞",
        CellType::Wall => "墙",
        CellType::Bomb => "炸弹",
        CellType::HighValue => "高价值单位",
    }
}

fn scrollable_pattern_list(ui: &mut egui::Ui, selected: &mut Option<String>, names: Vec<&str>) {
    for name in names {
        if ui
            .selectable_value(selected, Some(name.to_string()), name)
            .clicked()
        {}
    }
}
