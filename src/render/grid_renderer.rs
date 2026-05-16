use bevy::prelude::*;

use crate::grid::{CellType, Direction, Faction, Grid, GridCoord};
use crate::player::deploy::{DeployPhase, DragDeployState};
use crate::input::drag_drop::get_pattern_cells;
use crate::state::{AppState, DeploymentZoneData, SimulatorState};

/// 网格单元格实体标记组件
#[derive(Component)]
pub struct GridCell {
    pub x: usize,
    pub y: usize,
}

/// 预览幽灵单元格标记
#[derive(Component)]
pub struct GhostCell;

/// 部署区域单元格标记
#[derive(Component)]
pub struct ZoneCell;

/// 网格参数常量（渲染与输入系统共用）
pub const CELL_SIZE: f32 = 15.0;
pub const HUD_OFFSET: f32 = 20.0;

/// 将窗口像素坐标转换为网格坐标
pub fn screen_to_grid(cursor: Vec2, window: &Window, grid: &Grid) -> Option<GridCoord> {
    let win_w = window.resolution.width();
    let win_h = window.resolution.height();

    let world_x = cursor.x - win_w / 2.0;
    let world_y = win_h / 2.0 - cursor.y;

    let offset_x = -(grid.width as f32) * CELL_SIZE / 2.0;
    let offset_y = (grid.height as f32) * CELL_SIZE / 2.0 - HUD_OFFSET;

    let gx = (world_x - offset_x - CELL_SIZE / 2.0) / CELL_SIZE;
    let gy = (offset_y - world_y - CELL_SIZE / 2.0) / CELL_SIZE;

    if gx >= 0.0 && gy >= 0.0 {
        let x = gx as usize;
        let y = gy as usize;
        if grid.in_bounds(GridCoord::new(x, y)) {
            return Some(GridCoord::new(x, y));
        }
    }
    None
}

/// 网格渲染系统：为每个非空单元格生成/更新 sprite
pub fn grid_render_system(
    mut commands: Commands,
    grid: Res<Grid>,
    cell_query: Query<(Entity, &GridCell), (Without<GhostCell>, Without<ZoneCell>)>,
) {
    for (entity, _) in cell_query.iter() {
        commands.entity(entity).despawn();
    }

    let grid_w = grid.width as f32;
    let grid_h = grid.height as f32;

    let offset_x = -grid_w * CELL_SIZE / 2.0;
    let offset_y = grid_h * CELL_SIZE / 2.0 - HUD_OFFSET;

    for x in 0..grid.width {
        for y in 0..grid.height {
            let ct = grid.cells[x][y];
            if ct == CellType::Empty {
                continue;
            }

            let color = match ct {
                CellType::Normal(Faction::Red) => Color::srgb(1.0, 0.2, 0.2),
                CellType::Normal(Faction::Blue) => Color::srgb(0.2, 0.4, 1.0),
                CellType::Wall => Color::srgb(0.4, 0.4, 0.4),
                CellType::Bomb => Color::srgb(1.0, 0.5, 0.0),
                CellType::HighValue => Color::srgb(1.0, 0.8, 0.0),
                _ => continue,
            };

            let position = Vec3::new(
                offset_x + x as f32 * CELL_SIZE + CELL_SIZE / 2.0,
                offset_y - y as f32 * CELL_SIZE - CELL_SIZE / 2.0,
                0.0,
            );

            commands.spawn((
                Sprite {
                    color,
                    custom_size: Some(Vec2::splat(CELL_SIZE)),
                    ..default()
                },
                Transform::from_translation(position),
                GridCell { x, y },
            ));
        }
    }
}

/// 部署区域渲染系统：显示灰白色半透明区域
pub fn zone_render_system(
    mut commands: Commands,
    zone_query: Query<Entity, With<ZoneCell>>,
    zone_data: Res<DeploymentZoneData>,
    grid: Res<Grid>,
    state: Res<State<AppState>>,
) {
    // 清除旧区域实体
    for entity in zone_query.iter() {
        commands.entity(entity).despawn();
    }

    // 仅在模拟器和部署状态下渲染区域
    if *state.get() != AppState::Simulator && *state.get() != AppState::Deployment {
        return;
    }

    if zone_data.zone.is_empty() {
        return;
    }

    let grid_w = grid.width as f32;
    let grid_h = grid.height as f32;
    let offset_x = -grid_w * CELL_SIZE / 2.0;
    let offset_y = grid_h * CELL_SIZE / 2.0 - HUD_OFFSET;

    for coord in &zone_data.zone {
        if !grid.in_bounds(*coord) {
            continue;
        }

        let position = Vec3::new(
            offset_x + coord.x as f32 * CELL_SIZE + CELL_SIZE / 2.0,
            offset_y - coord.y as f32 * CELL_SIZE - CELL_SIZE / 2.0,
            -1.0, // 在普通格子下层显示
        );

        commands.spawn((
            Sprite {
                color: Color::srgba(0.85, 0.85, 0.92, 0.35),
                custom_size: Some(Vec2::splat(CELL_SIZE)),
                ..default()
            },
            Transform::from_translation(position),
            ZoneCell,
        ));
    }
}

/// 预览幽灵渲染系统：显示部署图案的半透明预览（含区域有效性颜色反馈）
pub fn ghost_preview_system(
    mut commands: Commands,
    ghost_query: Query<Entity, With<GhostCell>>,
    drag_state: Res<DragDeployState>,
    grid: Res<Grid>,
    zone_data: Res<DeploymentZoneData>,
    state: Res<State<AppState>>,
    sim_state: Res<State<SimulatorState>>,
) {
    // 清除旧幽灵
    for entity in ghost_query.iter() {
        commands.entity(entity).despawn();
    }

    let is_deploying = *state.get() == AppState::Deployment
        || (*state.get() == AppState::Simulator && *sim_state.get() == SimulatorState::DeploymentTest);

    if !is_deploying {
        return;
    }

    let unit = match drag_state.unit_type {
        Some(u) => u,
        None => return,
    };

    let dir = drag_state.rotation_dir.unwrap_or(Direction::Right);

    let preview_pos = match drag_state.phase {
        DeployPhase::Rotating => drag_state.start_pos,
        _ => drag_state.current_pos,
    };

    let pos = match preview_pos {
        Some(p) => p,
        None => return,
    };

    let cells = get_pattern_cells(unit, pos, dir);

    let grid_w = grid.width as f32;
    let grid_h = grid.height as f32;
    let offset_x = -grid_w * CELL_SIZE / 2.0;
    let offset_y = grid_h * CELL_SIZE / 2.0 - HUD_OFFSET;

    // 判断图案是否完全在有效区域内
    let zone_valid = if zone_data.zone.is_empty() {
        // 无区域限制时，只检查 in_bounds 和 is_empty
        cells.iter().all(|(c, _)| grid.in_bounds(*c) && grid.is_empty(*c))
    } else {
        // 有区域限制时，必须所有格子都在区域内且为空
        cells.iter().all(|(c, _)| {
            grid.in_bounds(*c) && grid.is_empty(*c) && zone_data.zone.contains(c)
        })
    };

    let alpha = if drag_state.phase == DeployPhase::Rotating { 0.7 } else { 0.4 };

    // 绿色 = 有效, 红色 = 无效
    let color = if zone_valid {
        Color::srgba(0.2, 1.0, 0.3, alpha)
    } else {
        Color::srgba(1.0, 0.2, 0.2, alpha)
    };

    for (coord, _) in &cells {
        if !grid.in_bounds(*coord) {
            continue;
        }

        let position = Vec3::new(
            offset_x + coord.x as f32 * CELL_SIZE + CELL_SIZE / 2.0,
            offset_y - coord.y as f32 * CELL_SIZE - CELL_SIZE / 2.0,
            1.0,
        );

        commands.spawn((
            Sprite {
                color,
                custom_size: Some(Vec2::splat(CELL_SIZE * 0.9)),
                ..default()
            },
            Transform::from_translation(position),
            GhostCell,
        ));
    }
}
