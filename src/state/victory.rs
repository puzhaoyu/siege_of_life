use bevy::prelude::*;
use bevy::ecs::system::SystemParam;

use crate::grid::{Grid, GridCoord};
use crate::level::data::SaveData;
use crate::level::loader::LevelRegistry;
use crate::level::progression;
use crate::state::{CurrentLevelId, DeploymentZoneData, SimulatorSnapshot};

/// 宝藏发光动画 6 帧 × 0.06s，略留余量再弹出通关
pub const VICTORY_SETTLE_DELAY_SECS: f32 = 0.42;

/// 延迟结算通关（等待特效播完）
#[derive(Resource, Default)]
pub struct PendingVictory {
    pub kind: Option<PendingVictoryKind>,
    timer: Option<Timer>,
}

#[derive(Clone, Copy)]
pub enum PendingVictoryKind {
    Level,
    Trial,
}

impl PendingVictory {
    pub fn is_pending(&self) -> bool {
        self.kind.is_some()
    }

    pub fn schedule(&mut self, kind: PendingVictoryKind) {
        self.kind = Some(kind);
        self.timer = Some(Timer::from_seconds(
            VICTORY_SETTLE_DELAY_SECS,
            TimerMode::Once,
        ));
    }

    pub fn clear(&mut self) {
        self.kind = None;
        self.timer = None;
    }
}

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

/// 闯关失败遮罩（关卡 / 模拟器试玩）
#[derive(Resource, Default)]
pub struct GameplayDefeatOverlay {
    pub kind: Option<DefeatKind>,
}

#[derive(Clone, Copy)]
pub enum DefeatKind {
    Level,
    Trial,
}

impl GameplayDefeatOverlay {
    pub fn is_active(&self) -> bool {
        self.kind.is_some()
    }

    pub fn clear(&mut self) {
        self.kind = None;
    }
}

pub fn trigger_level_defeat(overlay: &mut GameplayDefeatOverlay) {
    overlay.kind = Some(DefeatKind::Level);
}

pub fn trigger_trial_defeat(overlay: &mut GameplayDefeatOverlay) {
    overlay.kind = Some(DefeatKind::Trial);
}

pub fn any_gameplay_overlay_active(
    victory: &GameplayVictoryOverlay,
    defeat: &GameplayDefeatOverlay,
) -> bool {
    victory.is_active() || defeat.is_active()
}

/// 合并胜利/失败遮罩查询，避免系统参数过多
#[derive(SystemParam)]
pub struct GameplayOverlayState<'w> {
    pub victory: Res<'w, GameplayVictoryOverlay>,
    pub defeat: Res<'w, GameplayDefeatOverlay>,
}

impl GameplayOverlayState<'_> {
    pub fn blocks_input(&self) -> bool {
        any_gameplay_overlay_active(&self.victory, &self.defeat)
    }
}

/// 等待特效播完后触发通关 UI
pub fn pending_victory_system(
    time: Res<Time>,
    mut pending: ResMut<PendingVictory>,
    mut overlay: ResMut<GameplayVictoryOverlay>,
    current_level_id: Res<CurrentLevelId>,
    mut save_data: ResMut<SaveData>,
    registry: Res<LevelRegistry>,
) {
    if pending.kind.is_none() {
        return;
    }

    let Some(timer) = pending.timer.as_mut() else {
        pending.clear();
        return;
    };

    timer.tick(time.delta());
    if !timer.finished() {
        return;
    }

    match pending.kind.take() {
        Some(PendingVictoryKind::Level) => {
            trigger_level_victory(&mut overlay, &current_level_id, &mut save_data, &registry);
        }
        Some(PendingVictoryKind::Trial) => {
            trigger_trial_victory(&mut overlay);
        }
        None => {}
    }
    pending.timer = None;
}

/// 若本步摧毁了高价值单位，则延迟结算以便播放动画
pub fn resolve_high_value_victory(
    destroyed_high_values: &[GridCoord],
    pending: &mut PendingVictory,
    overlay: &mut GameplayVictoryOverlay,
    kind: PendingVictoryKind,
    current_level_id: &CurrentLevelId,
    save_data: &mut SaveData,
    registry: &LevelRegistry,
) {
    if destroyed_high_values.is_empty() {
        match kind {
            PendingVictoryKind::Level => {
                trigger_level_victory(overlay, current_level_id, save_data, registry);
            }
            PendingVictoryKind::Trial => {
                trigger_trial_victory(overlay);
            }
        }
    } else {
        pending.schedule(kind);
    }
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
