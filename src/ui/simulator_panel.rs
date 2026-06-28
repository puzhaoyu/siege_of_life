use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::grid::{CellType, Faction};
use crate::gol::engine;
use crate::patterns;
use crate::ui::level_io::{
    open_save_modal, request_open_browse, show_save_level_modal, SimulatorEditingParams,
};
use crate::player::deploy::DragDeployState;
use crate::player::resources::DeploymentResources;
use crate::state::victory::GameplayOverlayState;
use crate::state::{
    AppState, EvolutionConfig, SelectedElement, SelectedPattern, SimulatorSnapshot, SimulatorState,
};
use crate::ui::common::show_confirm_dialog;

/// 模拟器侧面板
pub fn simulator_panel_ui(
    mut contexts: EguiContexts,
    state: Res<State<AppState>>,
    sim_state: Res<State<SimulatorState>>,
    mut evo_config: ResMut<EvolutionConfig>,
    mut selected_element: ResMut<SelectedElement>,
    mut selected_pattern: ResMut<SelectedPattern>,
    mut next_state: ResMut<NextState<AppState>>,
    mut next_sim_state: ResMut<NextState<SimulatorState>>,
    mut deploy_res: ResMut<DeploymentResources>,
    mut snapshot: ResMut<SimulatorSnapshot>,
    mut drag_state: ResMut<DragDeployState>,
    mut editing: SimulatorEditingParams,
    overlay: GameplayOverlayState,
) {
    if *state.get() != AppState::Simulator {
        return;
    }

    if overlay.blocks_input() {
        return;
    }

    // 部署测试模式由 deployment_panel_ui 处理，不显示本面板
    if *sim_state.get() == SimulatorState::DeploymentTest {
        return;
    }

    let ctx = contexts.ctx_mut();

    show_save_level_modal(
        ctx,
        editing.file_dialog.as_mut(),
        editing.level_meta.as_mut(),
        &editing.grid,
        &editing.zone_data,
        &editing.deploy_config,
        editing.dialog.as_mut(),
    );

    // 弹出对话框
    let show_msg = editing.dialog.0.clone();
    if let Some(ref msg) = show_msg {
        let mut closed = false;
        crate::ui::common::show_dialog(ctx, msg, || {
            closed = true;
        });
        if closed {
            editing.dialog.0 = None;
        }
    }

    // 右侧面板（固定宽度，避免 egui 每帧按内容重算宽度导致抖动）
    egui::SidePanel::right("simulator_panel")
        .default_width(260.0)
        .width_range(260.0..=260.0)
        .resizable(false)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.set_width(248.0);
                    draw_simulator_panel_contents(
                        ui,
                        ctx,
                        *sim_state.get(),
                        &mut evo_config,
                        &mut selected_element,
                        &mut selected_pattern,
                        &mut next_state,
                        &mut next_sim_state,
                        &mut deploy_res,
                        &mut snapshot,
                        &mut drag_state,
                        &mut editing,
                    );
                });
        });
}

fn draw_simulator_panel_contents(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    sim_state: SimulatorState,
    evo_config: &mut EvolutionConfig,
    selected_element: &mut SelectedElement,
    selected_pattern: &mut SelectedPattern,
    next_state: &mut NextState<AppState>,
    next_sim_state: &mut NextState<SimulatorState>,
    deploy_res: &mut DeploymentResources,
    snapshot: &mut SimulatorSnapshot,
    drag_state: &mut DragDeployState,
    editing: &mut SimulatorEditingParams,
) {
            ui.heading("模拟器");

            ui.add_space(8.0);

            // 当前模式
            let mode_label = match sim_state {
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
            if sim_state == SimulatorState::Editing {
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
                                editing.eraser.active = false;
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
                        {
                            editing.eraser.active = false;
                        }
                        scrollable_pattern_list(
                            ui,
                            &mut selected_pattern.0,
                            patterns::still_life_names(),
                            || editing.eraser.active = false,
                        );
                        ui.separator();
                        scrollable_pattern_list(
                            ui,
                            &mut selected_pattern.0,
                            patterns::oscillator_names(),
                            || editing.eraser.active = false,
                        );
                        ui.separator();
                        scrollable_pattern_list(
                            ui,
                            &mut selected_pattern.0,
                            patterns::deployable_names(),
                            || editing.eraser.active = false,
                        );
                    });

                ui.add_space(8.0);
                ui.separator();

                // ---- 橡皮擦 ----
                ui.label(egui::RichText::new("橡皮擦").strong());
                ui.add_space(4.0);

                let eraser_label = if editing.eraser.active {
                    "橡皮擦: 开启"
                } else {
                    "橡皮擦: 关闭"
                };
                if ui.button(eraser_label).clicked() {
                    editing.eraser.active = !editing.eraser.active;
                    if editing.eraser.active {
                        editing.zone_brush.active = false;
                        selected_pattern.0 = None;
                    }
                }

                ui.horizontal(|ui| {
                    ui.label("半径:");
                    let mut radius = editing.eraser.radius as i32;
                    if ui.add(egui::Slider::new(&mut radius, 1..=5)).changed() {
                        editing.eraser.radius = radius as u32;
                    }
                });

                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("左键拖动擦除任意方块")
                        .weak()
                        .size(11.0),
                );

                ui.add_space(8.0);
                ui.separator();

                // ---- 部署设置 ----
                ui.label(egui::RichText::new("试玩部署设置").strong());
                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    ui.label("滑翔机:");
                    let mut gliders = editing.deploy_config.max_gliders as i32;
                    if ui
                        .add(egui::DragValue::new(&mut gliders).range(0..=10).speed(0.1))
                        .changed()
                    {
                        editing.deploy_config.max_gliders = gliders.max(0) as u32;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("轻型飞船:");
                    let mut lwss = editing.deploy_config.max_lwss as i32;
                    if ui
                        .add(egui::DragValue::new(&mut lwss).range(0..=5).speed(0.1))
                        .changed()
                    {
                        editing.deploy_config.max_lwss = lwss.max(0) as u32;
                    }
                });

                ui.add_space(8.0);
                ui.separator();

                // ---- 部署区域编辑 ----
                ui.label(egui::RichText::new("部署区域").strong());
                ui.add_space(4.0);

                let brush_label = if editing.zone_brush.active {
                    "画笔: 开启"
                } else {
                    "画笔: 关闭"
                };
                if ui.button(brush_label).clicked() {
                    editing.zone_brush.active = !editing.zone_brush.active;
                    if editing.zone_brush.active {
                        editing.eraser.active = false;
                    }
                }

                ui.horizontal(|ui| {
                    ui.label("画笔大小:");
                    let mut size = editing.zone_brush.size as i32;
                    if ui.add(egui::Slider::new(&mut size, 1..=5)).changed() {
                        editing.zone_brush.size = size as u32;
                    }
                });

                ui.label(format!("区域格数: {}", editing.zone_data.zone.len()));

                if ui.button("清除区域").clicked() {
                    editing.zone_data.zone.clear();
                }

                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("左键绘制部署区域")
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
                    // 保存进入试玩前的关卡（网格 + 部署区域）
                    snapshot.grid = Some(editing.grid.snapshot());
                    snapshot.zone = Some(editing.zone_data.zone.clone());
                    // 关闭区域画笔
                    editing.zone_brush.active = false;
                    // 初始化部署资源
                    deploy_res.remaining_gliders = editing.deploy_config.max_gliders;
                    deploy_res.remaining_lwss = editing.deploy_config.max_lwss;
                    deploy_res.deployed_this_round = false;
                    evo_config.current_step = 0;
                    evo_config.is_paused = true;
                    evo_config.timer = 0.0;
                    // 重置拖拽状态
                    *drag_state = DragDeployState::default();
                    // 进入部署测试模式
                    next_sim_state.set(SimulatorState::DeploymentTest);
                }

                ui.add_space(8.0);
                ui.separator();
            }

            // ---- 演化控制 (试玩模式) ----
            if sim_state == SimulatorState::TrialPlay {
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
                        let _step_result = engine::evolution_step(&mut editing.grid);
                        engine::check_high_value_destruction(&mut editing.grid);
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

                ui.label(format!(
                    "已演化: {}/{} 步",
                    evo_config.current_step, evo_config.steps_per_deployment
                ));

                ui.add_space(8.0);
                ui.separator();
            }

            // ---- 返回编辑按钮 (部署测试 / 试玩模式) ----
            if sim_state == SimulatorState::DeploymentTest
                || sim_state == SimulatorState::TrialPlay
            {
                ui.add_space(4.0);
                if ui
                    .add_sized(
                        egui::vec2(180.0, 36.0),
                        egui::Button::new(egui::RichText::new("返回编辑").size(15.0)),
                    )
                    .clicked()
                {
                    next_sim_state.set(SimulatorState::Editing);
                }

                ui.add_space(8.0);
                ui.separator();
            }

            // ---- 辅助功能 (仅编辑模式) ----
            if sim_state == SimulatorState::Editing {
                ui.label(egui::RichText::new("辅助").strong());

                if ui.button("清空全部").clicked() {
                    show_confirm_dialog(
                        ctx,
                        "确认清空",
                        "确定要清除网格上所有内容吗？",
                        || {
                            editing.grid.clear();
                        },
                        || {},
                    );
                }

                ui.add_space(8.0);
                ui.separator();

                // ---- 保存/打开 ----
                ui.label(egui::RichText::new("关卡文件").strong());

                if !editing.level_meta.name.is_empty() {
                    ui.label(
                        egui::RichText::new(format!("当前: {}", editing.level_meta.name))
                            .weak()
                            .size(11.0),
                    );
                }

                if ui.button("保存关卡…").clicked() {
                    open_save_modal(editing.file_dialog.as_mut(), &editing.level_meta);
                }

                if ui.button("打开关卡…").clicked() {
                    request_open_browse(editing.file_dialog.as_mut(), &editing.level_meta);
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
                *snapshot = SimulatorSnapshot::default();
                *drag_state = DragDeployState::default();
                editing.zone_brush.active = false;
                editing.eraser.active = false;
                next_sim_state.set(SimulatorState::Editing);
                next_state.set(AppState::MainMenu);
            }
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

fn scrollable_pattern_list(
    ui: &mut egui::Ui,
    selected: &mut Option<String>,
    names: Vec<&str>,
    mut on_select: impl FnMut(),
) {
    for name in names {
        if ui
            .selectable_value(selected, Some(name.to_string()), name)
            .clicked()
        {
            on_select();
        }
    }
}
