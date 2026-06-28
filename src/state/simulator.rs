use bevy::prelude::*;

use crate::grid::Grid;
use crate::gol::engine;
use crate::render::effects::{ClashEffectEvent, ExplosionEvent, TreasureGlowEvent};
use crate::player::deploy::DragDeployState;
use crate::player::resources::DeploymentResources;
use crate::state::victory::{
    any_gameplay_overlay_active, resolve_high_value_victory, trigger_trial_defeat,
    GameplayDefeatOverlay, GameplayVictoryOverlay, PendingVictory, PendingVictoryKind,
};
use crate::state::{AppState, CurrentLevelId, DeploymentZoneData, EvolutionConfig, SimulatorSnapshot, SimulatorState};

/// 模拟器初始化：清空网格
pub fn enter_simulator(
    mut grid: ResMut<Grid>,
    mut snapshot: ResMut<SimulatorSnapshot>,
    mut pending_victory: ResMut<PendingVictory>,
    mut defeat_overlay: ResMut<GameplayDefeatOverlay>,
) {
    grid.clear();
    *snapshot = SimulatorSnapshot::default();
    pending_victory.clear();
    defeat_overlay.clear();
}

/// 从试玩/部署返回编辑时，恢复进入试玩前的关卡内容
pub fn restore_simulator_editing_snapshot(
    mut snapshot: ResMut<SimulatorSnapshot>,
    mut grid: ResMut<Grid>,
    mut zone_data: ResMut<DeploymentZoneData>,
    mut evo_config: ResMut<EvolutionConfig>,
    mut drag_state: ResMut<DragDeployState>,
) {
    let Some(saved_grid) = snapshot.grid.take() else {
        return;
    };

    grid.restore(&saved_grid);
    if let Some(saved_zone) = snapshot.zone.take() {
        zone_data.zone = saved_zone;
    }

    evo_config.is_paused = true;
    evo_config.current_step = 0;
    evo_config.timer = 0.0;
    *drag_state = DragDeployState::default();
}

/// 模拟器演化系统（仅在 TrialPlay 状态运行，与主线相同：每轮固定步数后暂停等待部署）
pub fn simulator_evolution_system(
    time: Res<Time>,
    mut grid: ResMut<Grid>,
    mut evo_config: ResMut<EvolutionConfig>,
    mut deploy_res: ResMut<DeploymentResources>,
    state: Res<State<AppState>>,
    sim_state: Res<State<SimulatorState>>,
    mut next_sim_state: ResMut<NextState<SimulatorState>>,
    mut victory_overlay: ResMut<GameplayVictoryOverlay>,
    mut defeat_overlay: ResMut<GameplayDefeatOverlay>,
    mut pending_victory: ResMut<PendingVictory>,
    current_level_id: Res<CurrentLevelId>,
    mut save_data: ResMut<crate::level::data::SaveData>,
    registry: Res<crate::level::loader::LevelRegistry>,
    mut ev_explosion: EventWriter<ExplosionEvent>,
    mut ev_treasure_glow: EventWriter<TreasureGlowEvent>,
    mut ev_clash: EventWriter<ClashEffectEvent>,
) {
    if *state.get() != AppState::Simulator {
        return;
    }
    if *sim_state.get() != SimulatorState::TrialPlay {
        return;
    }
    if any_gameplay_overlay_active(&victory_overlay, &defeat_overlay)
        || pending_victory.is_pending()
        || evo_config.is_paused
    {
        return;
    }

    evo_config.timer += time.delta_secs() * 1000.0;

    while evo_config.timer >= evo_config.speed_ms {
        evo_config.timer -= evo_config.speed_ms;
        evo_config.current_step += 1;

        let step_result = engine::evolution_step(&mut grid);
        let destroyed_high_values = engine::check_high_value_destruction(&mut grid);

        if !step_result.bomb_result.triggered_bombs.is_empty() {
            ev_explosion.send(ExplosionEvent {
                positions: step_result.bomb_result.triggered_bombs.clone(),
            });
        }
        if !destroyed_high_values.is_empty() {
            ev_treasure_glow.send(TreasureGlowEvent {
                positions: destroyed_high_values.clone(),
            });
        }
        if !step_result.clash_positions.is_empty() {
            ev_clash.send(ClashEffectEvent {
                positions: step_result.clash_positions,
            });
        }

        if engine::all_high_values_destroyed(&grid) {
            evo_config.is_paused = true;
            resolve_high_value_victory(
                &destroyed_high_values,
                &mut pending_victory,
                &mut victory_overlay,
                PendingVictoryKind::Trial,
                &current_level_id,
                &mut save_data,
                &registry,
            );
            break;
        }

        if evo_config.current_step >= evo_config.steps_per_deployment {
            evo_config.is_paused = true;
            if deploy_res.remaining_gliders == 0 && deploy_res.remaining_lwss == 0 {
                trigger_trial_defeat(&mut defeat_overlay);
            } else {
                deploy_res.deployed_this_round = false;
                next_sim_state.set(SimulatorState::DeploymentTest);
            }
            break;
        }
    }
}
