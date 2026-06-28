use bevy::prelude::*;

use crate::gol::engine;
use crate::grid::Grid;
use crate::level::data::SaveData;
use crate::player::resources::DeploymentResources;
use crate::state::victory::{
    trigger_level_defeat, trigger_level_victory, GameplayDefeatOverlay, GameplayVictoryOverlay,
};
use crate::state::{AppState, CurrentJudgment, CurrentLevelId, EvolutionConfig, JudgmentResult};

/// 进入判定阶段
pub fn enter_judgment(commands: &mut Commands, grid: &Grid) {
    let all_destroyed = engine::all_high_values_destroyed(grid);
    let result = if all_destroyed {
        JudgmentResult::Victory
    } else {
        let _has_remaining = {
            // 无法在此直接获取 DeploymentResources，由 judgment_system 处理 ContinueDeploy
            true
        };
        JudgmentResult::ContinueDeploy
    };
    commands.insert_resource(CurrentJudgment(Some(result)));
    commands.insert_resource(NextState::Pending(AppState::Judgment));
}

/// 判定阶段系统：根据结果自动转换状态
pub fn judgment_system(
    mut next_state: ResMut<NextState<AppState>>,
    deploy_res: Res<DeploymentResources>,
    current_level_id: Res<CurrentLevelId>,
    mut save_data: ResMut<SaveData>,
    registry: Res<crate::level::loader::LevelRegistry>,
    mut victory_overlay: ResMut<GameplayVictoryOverlay>,
    mut defeat_overlay: ResMut<GameplayDefeatOverlay>,
    mut evo_config: ResMut<EvolutionConfig>,
    mut judgment: ResMut<CurrentJudgment>,
) {
    let result = match judgment.0.take() {
        Some(r) => r,
        None => return,
    };

    match result {
        JudgmentResult::Victory => {
            trigger_level_victory(
                &mut victory_overlay,
                &current_level_id,
                &mut save_data,
                &registry,
            );
            evo_config.is_paused = true;
            next_state.set(AppState::Evolution);
        }
        JudgmentResult::ContinueDeploy => {
            let has_remaining = deploy_res.remaining_gliders > 0 || deploy_res.remaining_lwss > 0;
            if has_remaining {
                next_state.set(AppState::Deployment);
            } else {
                trigger_level_defeat(&mut defeat_overlay);
                evo_config.is_paused = true;
                next_state.set(AppState::Evolution);
            }
        }
        JudgmentResult::Defeat => {
            trigger_level_defeat(&mut defeat_overlay);
            evo_config.is_paused = true;
            next_state.set(AppState::Evolution);
        }
    }
}
