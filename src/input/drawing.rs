use bevy::prelude::*;
use bevy_egui::EguiContexts;

use crate::grid::{CellType, Grid, GridCoord};
use crate::render::grid_renderer::screen_to_grid;
use crate::state::{AppState, DeploymentZoneData, SelectedElement, SelectedPattern, SimulatorState, ZoneBrushConfig};
use crate::patterns;

/// 模拟器绘制模式
#[derive(Resource, Clone, PartialEq, Eq)]
pub enum DrawMode {
    Draw,
    Erase,
}

impl Default for DrawMode {
    fn default() -> Self {
        DrawMode::Draw
    }
}

/// 绘制历史记录（用于撤销）
#[derive(Resource, Clone, Default)]
pub struct DrawHistory {
    pub history: Vec<Vec<(GridCoord, CellType)>>,
}

/// 模拟器绘制系统
pub fn drawing_system(
    mut grid: ResMut<Grid>,
    mut contexts: EguiContexts,
    state: Res<State<AppState>>,
    sim_state: Res<State<SimulatorState>>,
    selected_element: Res<SelectedElement>,
    selected_pattern: Res<SelectedPattern>,
    zone_brush: Res<ZoneBrushConfig>,
    mut zone_data: ResMut<DeploymentZoneData>,
    mut draw_mode: Local<DrawMode>,
    mut last_draw_coord: Local<Option<GridCoord>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    mut draw_history: Local<DrawHistory>,
    _next_state: ResMut<NextState<SimulatorState>>,
) {
    if contexts.ctx_mut().wants_pointer_input() {
        return;
    }

    if *state.get() != AppState::Simulator || *sim_state.get() != SimulatorState::Editing {
        *last_draw_coord = None;
        return;
    }

    let window = match windows.get_single() {
        Ok(w) => w,
        Err(_) => return,
    };

    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    let Some(coord) = screen_to_grid(cursor_pos, window, &grid) else {
        *last_draw_coord = None;
        return;
    };

    // 区域画笔模式
    if zone_brush.active {
        let is_left = mouse_input.pressed(MouseButton::Left);
        let is_right = mouse_input.pressed(MouseButton::Right);

        if !is_left && !is_right {
            *last_draw_coord = None;
            return;
        }

        if *last_draw_coord == Some(coord) {
            return;
        }
        *last_draw_coord = Some(coord);

        let brush_coords = get_brush_coords(coord, zone_brush.size, &grid);

        if is_left {
            // 左键添加区域
            for c in brush_coords {
                if !zone_data.zone.contains(&c) {
                    zone_data.zone.push(c);
                }
            }
        } else if is_right {
            // 右键移除区域
            for c in brush_coords {
                zone_data.zone.retain(|z| *z != c);
            }
        }
        return;
    }

    // 普通绘制模式
    if mouse_input.just_pressed(MouseButton::Right) {
        *draw_mode = DrawMode::Erase;
    }
    if mouse_input.just_released(MouseButton::Right) {
        *draw_mode = DrawMode::Draw;
    }

    let is_pressed = mouse_input.pressed(MouseButton::Left);
    let just_pressed = mouse_input.just_pressed(MouseButton::Left);

    if !is_pressed {
        *last_draw_coord = None;
        return;
    }

    if *last_draw_coord == Some(coord) && !just_pressed {
        return;
    }
    *last_draw_coord = Some(coord);

    let mut changed = Vec::new();

    if *draw_mode == DrawMode::Erase {
        changed.push((coord, CellType::Empty));
        grid.set(coord, CellType::Empty);
    } else if let Some(ref pattern_name) = selected_pattern.0 {
        if just_pressed {
            if let Some(cells) = patterns::get_pattern_by_name(pattern_name, coord) {
                for (c, ct) in &cells {
                    changed.push((*c, grid.get(*c)));
                    grid.set(*c, *ct);
                }
            }
        }
    } else {
        let element = selected_element.0;
        if grid.is_empty(coord) || element == CellType::Empty {
            changed.push((coord, grid.get(coord)));
            grid.set(coord, element);
        }
    }

    if !changed.is_empty() {
        draw_history.history.push(changed);
        if draw_history.history.len() > 100 {
            draw_history.history.remove(0);
        }
    }
}

/// 根据画笔大小获取影响的坐标列表
fn get_brush_coords(center: GridCoord, size: u32, grid: &Grid) -> Vec<GridCoord> {
    let mut coords = Vec::new();
    let radius = (size as isize - 1).max(0);

    for dx in -radius..=radius {
        for dy in -radius..=radius {
            let nx = center.x as isize + dx;
            let ny = center.y as isize + dy;
            if nx >= 0 && ny >= 0 {
                let coord = GridCoord::new(nx as usize, ny as usize);
                if grid.in_bounds(coord) {
                    coords.push(coord);
                }
            }
        }
    }
    coords
}

/// 撤销操作
pub fn undo_last_draw(
    mut grid: ResMut<Grid>,
    mut draw_history: Local<DrawHistory>,
) {
    if let Some(last_changes) = draw_history.history.pop() {
        for (coord, old_cell) in last_changes {
            grid.set(coord, old_cell);
        }
    }
}
