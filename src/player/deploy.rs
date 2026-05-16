use bevy::prelude::*;

use crate::grid::{Direction, Grid, GridCoord};
use crate::level::data::LevelData;
use crate::player::resources::DeploymentResources;
use crate::patterns;

/// 拖拽部署状态
#[derive(Resource, Clone)]
pub struct DragDeployState {
    pub active: bool,
    pub unit_type: Option<DeployUnitType>,
    pub start_pos: Option<GridCoord>,
    pub current_pos: Option<GridCoord>,
    pub rotation_dir: Option<Direction>,
    pub phase: DeployPhase,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DeployUnitType {
    Glider,
    LWSS,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DeployPhase {
    Idle,
    Dragging,
    Rotating,
}

impl Default for DragDeployState {
    fn default() -> Self {
        Self {
            active: false,
            unit_type: None,
            start_pos: None,
            current_pos: None,
            rotation_dir: None,
            phase: DeployPhase::Idle,
        }
    }
}

/// 验证部署位置是否合法
pub fn validate_deploy_position(
    grid: &Grid,
    unit: DeployUnitType,
    pos: GridCoord,
    dir: Direction,
    level_data: &LevelData,
) -> bool {
    // 获取图案占据的格子
    let cells = match unit {
        DeployUnitType::Glider => patterns::glider_red(pos, dir),
        DeployUnitType::LWSS => patterns::lwss_red(pos, dir),
    };

    for (coord, _) in &cells {
        // 必须在部署区域内
        if !level_data.deployment_zone.contains(coord) {
            return false;
        }
        // 不能与既有内容重叠（除非是空的）
        if !grid.is_empty(*coord) {
            return false;
        }
    }
    true
}

/// 执行部署放置
pub fn place_unit(
    grid: &mut Grid,
    unit: DeployUnitType,
    pos: GridCoord,
    dir: Direction,
    deploy_res: &mut DeploymentResources,
) -> bool {
    let cells = match unit {
        DeployUnitType::Glider => {
            if deploy_res.remaining_gliders == 0 {
                return false;
            }
            patterns::glider_red(pos, dir)
        }
        DeployUnitType::LWSS => {
            if deploy_res.remaining_lwss == 0 {
                return false;
            }
            patterns::lwss_red(pos, dir)
        }
    };

    for (coord, cell) in &cells {
        grid.set(*coord, *cell);
    }

    match unit {
        DeployUnitType::Glider => deploy_res.remaining_gliders -= 1,
        DeployUnitType::LWSS => deploy_res.remaining_lwss -= 1,
    }
    deploy_res.deployed_this_round = true;
    true
}
