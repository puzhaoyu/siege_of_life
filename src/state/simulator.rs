use bevy::prelude::*;

use crate::grid::Grid;
use crate::gol::engine;
use crate::state::victory::{trigger_trial_victory, GameplayVictoryOverlay};
use crate::player::deploy::DragDeployState;
use crate::player::resources::DeploymentResources;
use crate::state::{
    AppState, DeploymentZoneData, EvolutionConfig, SimulatorSnapshot, SimulatorState,
};

/// 模拟器初始化：清空网格
pub fn enter_simulator(mut grid: ResMut<Grid>, mut snapshot: ResMut<SimulatorSnapshot>) {
    grid.clear();
    *snapshot = SimulatorSnapshot::default();
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
) {
    if *state.get() != AppState::Simulator {
        return;
    }
    if *sim_state.get() != SimulatorState::TrialPlay {
        return;
    }
    if victory_overlay.is_active() || evo_config.is_paused {
        return;
    }

    evo_config.timer += time.delta_secs() * 1000.0;

    while evo_config.timer >= evo_config.speed_ms {
        evo_config.timer -= evo_config.speed_ms;
        evo_config.current_step += 1;

        let (_changed, _bomb_result) = engine::evolution_step(&mut grid);
        engine::check_high_value_destruction(&mut grid);

        if engine::all_high_values_destroyed(&grid) {
            evo_config.is_paused = true;
            trigger_trial_victory(&mut victory_overlay);
            break;
        }

        if evo_config.current_step >= evo_config.steps_per_deployment {
            evo_config.is_paused = true;
            deploy_res.deployed_this_round = false;
            next_sim_state.set(SimulatorState::DeploymentTest);
            break;
        }
    }
}
