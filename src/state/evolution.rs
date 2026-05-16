use bevy::prelude::*;

use crate::gol::engine;
use crate::grid::Grid;
use crate::state::{AppState, EvolutionConfig};
use crate::state::judgment::enter_judgment;

/// 演化阶段：每帧根据速度推进演化步
pub fn evolution_system(
    mut commands: Commands,
    time: Res<Time>,
    mut grid: ResMut<Grid>,
    mut evo_config: ResMut<EvolutionConfig>,
    state: Res<State<AppState>>,
) {
    if *state.get() != AppState::Evolution {
        return;
    }
    if evo_config.is_paused {
        return;
    }

    evo_config.timer += time.delta_secs() * 1000.0; // 转换为毫秒

    while evo_config.timer >= evo_config.speed_ms {
        evo_config.timer -= evo_config.speed_ms;
        evo_config.current_step += 1;

        // 执行一步演化
        let (_changed, _bomb_result) = engine::evolution_step(&mut grid);
        // 检测高价值单位摧毁
        engine::check_high_value_destruction(&mut grid);

        // 检查是否达到限制步数
        if evo_config.current_step >= evo_config.steps_per_deployment {
            evo_config.is_paused = true;
            enter_judgment(&mut commands, &grid);
            break;
        }
    }
}

/// 进入演化阶段时重置步数
pub fn enter_evolution(mut evo_config: ResMut<EvolutionConfig>) {
    evo_config.current_step = 0;
    evo_config.is_paused = false;
    evo_config.timer = 0.0;
}
