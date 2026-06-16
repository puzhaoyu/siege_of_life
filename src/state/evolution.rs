use bevy::prelude::*;
use bevy_egui::EguiContexts;

use crate::gol::engine;
use crate::grid::Grid;
use crate::render::effects::{ClashEffectEvent, ExplosionEvent, TreasureGlowEvent};
use crate::level::data::SaveData;
use crate::level::loader::LevelRegistry;
use crate::state::judgment::enter_judgment;
use crate::state::victory::{
    resolve_high_value_victory, PendingVictory, PendingVictoryKind, GameplayVictoryOverlay,
};
use crate::state::{AppState, CurrentLevelId, EvolutionConfig, SimulatorState};

/// 一轮演算结束原因
pub enum RoundEvolutionResult {
    Victory {
        destroyed_high_values: Vec<crate::grid::GridCoord>,
    },
    StepLimitReached,
}

/// 立即完成本轮剩余演算步数（跳过动画）
pub fn fast_forward_current_round(
    grid: &mut Grid,
    evo_config: &mut EvolutionConfig,
) -> RoundEvolutionResult {
    while evo_config.current_step < evo_config.steps_per_deployment {
        evo_config.current_step += 1;
        let _step_result = engine::evolution_step(grid);
        let destroyed_high_values = engine::check_high_value_destruction(grid);

        if engine::all_high_values_destroyed(grid) {
            evo_config.is_paused = true;
            evo_config.timer = 0.0;
            return RoundEvolutionResult::Victory {
                destroyed_high_values,
            };
        }
    }

    evo_config.is_paused = true;
    evo_config.timer = 0.0;
    RoundEvolutionResult::StepLimitReached
}

/// 演算中点击画面：跳过动画，直接完成本轮剩余步数
pub fn skip_evolution_on_click_system(
    mut contexts: EguiContexts,
    mouse_input: Res<ButtonInput<MouseButton>>,
    state: Res<State<AppState>>,
    sim_state: Res<State<SimulatorState>>,
    mut victory_overlay: ResMut<GameplayVictoryOverlay>,
    mut pending_victory: ResMut<PendingVictory>,
    mut commands: Commands,
    mut grid: ResMut<Grid>,
    mut evo_config: ResMut<EvolutionConfig>,
    current_level_id: Res<CurrentLevelId>,
    mut save_data: ResMut<SaveData>,
    registry: Res<LevelRegistry>,
    mut deploy_res: ResMut<crate::player::resources::DeploymentResources>,
    mut next_sim_state: ResMut<NextState<SimulatorState>>,
    mut ev_treasure_glow: EventWriter<TreasureGlowEvent>,
) {
    if victory_overlay.is_active() || pending_victory.is_pending() || evo_config.is_paused {
        return;
    }

    let is_level_evolving = *state.get() == AppState::Evolution;
    let is_trial_evolving = *state.get() == AppState::Simulator
        && *sim_state.get() == SimulatorState::TrialPlay;

    if !is_level_evolving && !is_trial_evolving {
        return;
    }

    if contexts.ctx_mut().wants_pointer_input() {
        return;
    }

    if !mouse_input.just_pressed(MouseButton::Left) {
        return;
    }

    match fast_forward_current_round(&mut grid, &mut evo_config) {
        RoundEvolutionResult::Victory {
            destroyed_high_values,
        } => {
            if !destroyed_high_values.is_empty() {
                ev_treasure_glow.send(TreasureGlowEvent {
                    positions: destroyed_high_values.clone(),
                });
            }
            if is_level_evolving {
                resolve_high_value_victory(
                    &destroyed_high_values,
                    &mut pending_victory,
                    &mut victory_overlay,
                    PendingVictoryKind::Level,
                    &current_level_id,
                    &mut save_data,
                    &registry,
                );
            } else {
                resolve_high_value_victory(
                    &destroyed_high_values,
                    &mut pending_victory,
                    &mut victory_overlay,
                    PendingVictoryKind::Trial,
                    &current_level_id,
                    &mut save_data,
                    &registry,
                );
            }
        }
        RoundEvolutionResult::StepLimitReached => {
            if is_level_evolving {
                enter_judgment(&mut commands, &grid);
            } else {
                deploy_res.deployed_this_round = false;
                next_sim_state.set(SimulatorState::DeploymentTest);
            }
        }
    }
}

/// 演化阶段：每帧根据速度推进演化步
pub fn evolution_system(
    mut commands: Commands,
    time: Res<Time>,
    mut grid: ResMut<Grid>,
    mut evo_config: ResMut<EvolutionConfig>,
    state: Res<State<AppState>>,
    mut victory_overlay: ResMut<GameplayVictoryOverlay>,
    mut pending_victory: ResMut<PendingVictory>,
    current_level_id: Res<CurrentLevelId>,
    mut save_data: ResMut<SaveData>,
    registry: Res<LevelRegistry>,
    mut ev_explosion: EventWriter<ExplosionEvent>,
    mut ev_treasure_glow: EventWriter<TreasureGlowEvent>,
    mut ev_clash: EventWriter<ClashEffectEvent>,
) {
    if *state.get() != AppState::Evolution {
        return;
    }
    if victory_overlay.is_active() || pending_victory.is_pending() || evo_config.is_paused {
        return;
    }

    evo_config.timer += time.delta_secs() * 1000.0; // 转换为毫秒

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
                PendingVictoryKind::Level,
                &current_level_id,
                &mut save_data,
                &registry,
            );
            break;
        }

        if evo_config.current_step >= evo_config.steps_per_deployment {
            evo_config.is_paused = true;
            enter_judgment(&mut commands, &grid);
            break;
        }
    }
}

/// 进入演化阶段时重置步数
pub fn enter_evolution(
    mut evo_config: ResMut<EvolutionConfig>,
    mut pending_victory: ResMut<PendingVictory>,
) {
    evo_config.current_step = 0;
    evo_config.is_paused = false;
    evo_config.timer = 0.0;
    pending_victory.clear();
}
