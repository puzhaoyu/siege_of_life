use bevy::prelude::*;
use bevy_egui::EguiContexts;

use crate::grid::{Direction, Grid, GridCoord};
use crate::level::loader::LevelRegistry;
use crate::player::deploy::{DeployPhase, DeployUnitType, DragDeployState, place_unit};
use crate::player::resources::DeploymentResources;
use crate::render::grid_renderer::screen_to_grid;
use crate::state::deployment::begin_evolution_after_deploy;
use crate::state::victory::GameplayOverlayState;
use crate::state::{AppState, CurrentLevelId, DeploymentZoneData, EvolutionConfig, SimulatorState};
use crate::patterns;

/// 部署交互系统：
/// Phase 1 (Idle + unit_type set): 图案跟随鼠标
/// Phase 2 (Rotating): 落点固定，鼠标拖动旋转（像旋钮），图案实时旋转
/// 点击确认放置
pub fn drag_drop_system(
    mut grid: ResMut<Grid>,
    mut contexts: EguiContexts,
    mut drag_state: ResMut<DragDeployState>,
    mut deploy_res: ResMut<DeploymentResources>,
    state: Res<State<AppState>>,
    sim_state: Res<State<SimulatorState>>,
    zone_data: Res<DeploymentZoneData>,
    _level_id: Res<CurrentLevelId>,
    _registry: Res<LevelRegistry>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    key_input: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    mut next_state: ResMut<NextState<AppState>>,
    mut next_sim_state: ResMut<NextState<SimulatorState>>,
    mut evo_config: ResMut<EvolutionConfig>,
    overlay: GameplayOverlayState,
) {
    if contexts.ctx_mut().wants_pointer_input() {
        return;
    }

    if overlay.blocks_input() {
        return;
    }

    // 判断是否处于部署状态（关卡部署 或 模拟器部署测试）
    let is_deploying = *state.get() == AppState::Deployment
        || (*state.get() == AppState::Simulator && *sim_state.get() == SimulatorState::DeploymentTest);

    if !is_deploying {
        // 不在部署状态时重置
        if drag_state.unit_type.is_some() {
            drag_state.phase = DeployPhase::Idle;
            drag_state.unit_type = None;
        }
        return;
    }

    let window = match windows.get_single() {
        Ok(w) => w,
        Err(_) => return,
    };

    let cursor_grid_pos = window
        .cursor_position()
        .and_then(|cursor| screen_to_grid(cursor, window, &grid));

    // ESC 取消当前操作
    if key_input.just_pressed(KeyCode::Escape) {
        if drag_state.unit_type.is_some() || drag_state.phase != DeployPhase::Idle {
            drag_state.phase = DeployPhase::Idle;
            drag_state.unit_type = None;
            drag_state.start_pos = None;
            drag_state.current_pos = None;
            drag_state.rotation_dir = None;
        } else {
            // ESC 退回上一级
            if *state.get() == AppState::Deployment {
                next_state.set(AppState::LevelSelect);
            } else {
                next_sim_state.set(SimulatorState::Editing);
            }
        }
        return;
    }

    match drag_state.phase {
        DeployPhase::Idle => {
            // 已选择单位 → 图案跟随鼠标移动
            if drag_state.unit_type.is_some() {
                drag_state.current_pos = cursor_grid_pos;

                // 点击网格 → 先验证区域有效性，再锁定落点进入旋转模式
                if mouse_input.just_pressed(MouseButton::Left) {
                    if let Some(pos) = cursor_grid_pos {
                        if let Some(unit) = drag_state.unit_type {
                            // 检查是否存在至少一个方向使图案完全在区域内
                            let can_place = if zone_data.zone.is_empty() {
                                true
                            } else {
                                [Direction::Up, Direction::Down, Direction::Left, Direction::Right]
                                    .iter()
                                    .any(|&d| {
                                        let cells = get_pattern_cells(unit, pos, d);
                                        cells.iter().all(|(c, _)| {
                                            grid.in_bounds(*c)
                                                && grid.is_empty(*c)
                                                && zone_data.zone.contains(c)
                                        })
                                    })
                            };
                            if can_place {
                                drag_state.start_pos = Some(pos);
                                drag_state.rotation_dir = Some(if let Some(cur) = cursor_grid_pos {
                                    direction_from_anchor_to_cursor(pos, cur)
                                } else {
                                    Direction::Right
                                });
                                drag_state.phase = DeployPhase::Rotating;
                            }
                            // 无效位置：不进入旋转模式，静默拒绝（红色预览已给出提示）
                        }
                    }
                }
            }
        }
        DeployPhase::Dragging => {
            // 兼容保留，直接转回 Idle
            drag_state.phase = DeployPhase::Idle;
        }
        DeployPhase::Rotating => {
            // 旋钮旋转: 鼠标相对落点方向 → 四向朝向（每 90° 一档）
            if let Some(anchor) = drag_state.start_pos {
                if let Some(current) = cursor_grid_pos {
                    drag_state.rotation_dir = Some(direction_from_anchor_to_cursor(anchor, current));
                    drag_state.current_pos = Some(current);
                }
            }

            // 点击确认放置
            if mouse_input.just_pressed(MouseButton::Left) {
                if let (Some(unit), Some(pos), Some(dir)) =
                    (drag_state.unit_type, drag_state.start_pos, drag_state.rotation_dir)
                {
                    let cells = get_pattern_cells(unit, pos, dir);
                    let all_valid = cells.iter().all(|(c, _)| {
                        if !grid.in_bounds(*c) || !grid.is_empty(*c) {
                            return false;
                        }
                        // 如果有部署区域限制，必须在区域内
                        if !zone_data.zone.is_empty() && !zone_data.zone.contains(c) {
                            return false;
                        }
                        true
                    });
                    if all_valid && place_unit(&mut grid, unit, pos, dir, &mut deploy_res) {
                        drag_state.phase = DeployPhase::Idle;
                        drag_state.unit_type = None;
                        drag_state.start_pos = None;
                        drag_state.current_pos = None;
                        drag_state.rotation_dir = None;
                        begin_evolution_after_deploy(
                            &state,
                            &sim_state,
                            &mut next_state,
                            &mut next_sim_state,
                            &mut evo_config,
                        );
                        return;
                    }
                }
                // 重置
                drag_state.phase = DeployPhase::Idle;
                drag_state.unit_type = None;
                drag_state.start_pos = None;
                drag_state.current_pos = None;
                drag_state.rotation_dir = None;
            }
        }
    }
}

/// 根据锚点相对鼠标位置推断四向朝向（0°/90°/180°/270°）
fn direction_from_anchor_to_cursor(anchor: GridCoord, cursor: GridCoord) -> Direction {
    let dx = cursor.x as isize - anchor.x as isize;
    let dy = cursor.y as isize - anchor.y as isize;

    if dx == 0 && dy == 0 {
        return Direction::Right;
    }

    let angle = (dy as f32).atan2(dx as f32);
    const FRAC_PI_4: f32 = std::f32::consts::FRAC_PI_4;

    if angle >= -FRAC_PI_4 && angle < FRAC_PI_4 {
        Direction::Right
    } else if angle >= FRAC_PI_4 && angle < 3.0 * FRAC_PI_4 {
        Direction::Down
    } else if angle < -FRAC_PI_4 && angle >= -3.0 * FRAC_PI_4 {
        Direction::Up
    } else {
        Direction::Left
    }
}

/// 获取图案细胞列表
pub fn get_pattern_cells(unit: DeployUnitType, pos: GridCoord, dir: Direction) -> Vec<(GridCoord, crate::grid::CellType)> {
    match unit {
        DeployUnitType::Glider => patterns::glider_red(pos, dir),
        DeployUnitType::LWSS => patterns::lwss_red(pos, dir),
    }
}

/// 从部署面板选择单位，进入跟随模式
pub fn start_drag(drag_state: &mut DragDeployState, unit: DeployUnitType) {
    drag_state.unit_type = Some(unit);
    drag_state.phase = DeployPhase::Idle;
    drag_state.start_pos = None;
    drag_state.current_pos = None;
    drag_state.rotation_dir = Some(Direction::Right);
}
