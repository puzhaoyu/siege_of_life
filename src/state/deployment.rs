use bevy::prelude::*;

use crate::grid::{CellType, Grid};
use crate::level::data::LevelData;
use crate::level::loader::LevelRegistry;
use crate::player::deploy::DragDeployState;
use crate::state::{CurrentLevelId, DeploymentZoneData};

/// 进入部署阶段：重置拖拽状态，加载部署区域
pub fn enter_deployment(
    mut drag_state: ResMut<DragDeployState>,
    mut zone_data: ResMut<DeploymentZoneData>,
    level_id: Res<CurrentLevelId>,
    registry: Res<LevelRegistry>,
) {
    drag_state.phase = crate::player::deploy::DeployPhase::Idle;
    drag_state.unit_type = None;
    drag_state.rotation_dir = None;
    drag_state.start_pos = None;
    drag_state.current_pos = None;

    // 从关卡数据加载部署区域
    if let Some(ref id) = level_id.0 {
        if let Some(data) = registry.get(id) {
            zone_data.zone = data.deployment_zone.clone();
        } else {
            zone_data.zone.clear();
        }
    } else {
        zone_data.zone.clear();
    }
}

pub fn load_level_to_grid(grid: &mut Grid, data: &LevelData) {
    grid.clear();
    grid.width = data.width;
    grid.height = data.height;
    grid.cells = vec![vec![CellType::Empty; data.height]; data.width];

    // data.initial_cells[y][x] = row y, column x
    // grid.cells[x][y]     = column x, row y
    for y in 0..data.height.min(data.initial_cells.len()) {
        for x in 0..data.width.min(data.initial_cells[y].len()) {
            grid.cells[x][y] = data.initial_cells[y][x];
        }
    }
}
