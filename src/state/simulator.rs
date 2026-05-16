use bevy::prelude::*;

use crate::grid::Grid;
use crate::gol::engine;
use crate::state::{AppState, EvolutionConfig, SimulatorState};

/// 模拟器初始化：清空网格或加载已有关卡
pub fn enter_simulator(mut grid: ResMut<Grid>) {
    grid.clear();
}

/// 模拟器演化系统（仅在 TrialPlay 状态运行）
pub fn simulator_evolution_system(
    time: Res<Time>,
    mut grid: ResMut<Grid>,
    mut evo_config: ResMut<EvolutionConfig>,
    state: Res<State<AppState>>,
    sim_state: Res<State<SimulatorState>>,
) {
    if *state.get() != AppState::Simulator {
        return;
    }
    if *sim_state.get() != SimulatorState::TrialPlay {
        return;
    }
    if evo_config.is_paused {
        return;
    }

    evo_config.timer += time.delta_secs() * 1000.0;

    while evo_config.timer >= evo_config.speed_ms {
        evo_config.timer -= evo_config.speed_ms;
        evo_config.current_step += 1;

        let (_changed, _bomb_result) = engine::evolution_step(&mut grid);
        engine::check_high_value_destruction(&mut grid);
    }
}
