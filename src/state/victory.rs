use bevy::prelude::*;

use crate::grid::Grid;
use crate::level::data::SaveData;
use crate::level::loader::LevelRegistry;
use crate::level::progression;
use crate::state::{CurrentLevelId, DeploymentZoneData, SimulatorSnapshot};

/// 闯关成功遮罩（关卡 / 模拟器试玩）
#[derive(Resource, Default)]
pub struct GameplayVictoryOverlay {
    pub kind: Option<VictoryKind>,
}

#[derive(Clone)]
pub enum VictoryKind {
    Level {
        has_next_level: bool,
    },
    Trial,
}

impl GameplayVictoryOverlay {
    pub fn is_active(&self) -> bool {
        self.kind.is_some()
    }

    pub fn clear(&mut self) {
        self.kind = None;
    }
}

pub fn save_level_progress(
    current_level_id: &CurrentLevelId,
    save_data: &mut SaveData,
    registry: &LevelRegistry,
) -> bool {
    let id = match &current_level_id.0 {
        Some(id) => id.clone(),
        None => return false,
    };

    if !save_data.completed_levels.contains(&id) {
        save_data.completed_levels.push(id.clone());
    }

    let idx = registry.ordered_ids.iter().position(|i| i == &id);
    let has_next = if let Some(idx) = idx {
        let next_idx = idx + 1;
        if next_idx < registry.ordered_ids.len() {
            let next_id = registry.ordered_ids[next_idx].clone();
            if !save_data.unlocked_levels.contains(&next_id) {
                save_data.unlocked_levels.push(next_id);
            }
            true
        } else {
            false
        }
    } else {
        false
    };

    progression::save_progress(save_data);
    has_next
}

pub fn trigger_level_victory(
    overlay: &mut GameplayVictoryOverlay,
    current_level_id: &CurrentLevelId,
    save_data: &mut SaveData,
    registry: &LevelRegistry,
) {
    let has_next = save_level_progress(current_level_id, save_data, registry);
    overlay.kind = Some(VictoryKind::Level { has_next_level: has_next });
}

pub fn trigger_trial_victory(overlay: &mut GameplayVictoryOverlay) {
    overlay.kind = Some(VictoryKind::Trial);
}

/// 从试玩快照恢复网格（不消耗快照，供「重新开始」使用）
pub fn apply_trial_snapshot(
    snapshot: &SimulatorSnapshot,
    grid: &mut Grid,
    zone_data: &mut DeploymentZoneData,
) -> bool {
    let Some(saved_grid) = &snapshot.grid else {
        return false;
    };
    grid.restore(saved_grid);
    if let Some(ref zone) = snapshot.zone {
        zone_data.zone = zone.clone();
    }
    true
}
