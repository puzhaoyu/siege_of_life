use bevy::prelude::*;

use crate::gol::engine;
use crate::grid::Grid;
use crate::level::data::SaveData;
use crate::level::progression;
use crate::player::resources::DeploymentResources;
use crate::state::{AppState, CurrentJudgment, CurrentLevelId, JudgmentResult};

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
    mut dialog: ResMut<crate::state::DialogMessage>,
    judgment: Res<CurrentJudgment>,
    deploy_res: Res<DeploymentResources>,
    current_level_id: Res<CurrentLevelId>,
    mut save_data: ResMut<SaveData>,
    registry: Res<crate::level::loader::LevelRegistry>,
) {
    let result = match &judgment.0 {
        Some(r) => r.clone(),
        None => return,
    };

    match result {
        JudgmentResult::Victory => {
            // 保存进度
            if let Some(ref id) = current_level_id.0 {
                if !save_data.completed_levels.contains(id) {
                    save_data.completed_levels.push(id.clone());
                }
                // 解锁下一关
                let idx = registry.ordered_ids.iter().position(|i| i == id);
                if let Some(idx) = idx {
                    let next_idx = idx + 1;
                    if next_idx < registry.ordered_ids.len() {
                        let next_id = registry.ordered_ids[next_idx].clone();
                        if !save_data.unlocked_levels.contains(&next_id) {
                            save_data.unlocked_levels.push(next_id);
                        }
                    }
                }
                progression::save_progress(&save_data);
            }

            dialog.0 = Some("胜利！所有高价值单位已被摧毁！".to_string());
            next_state.set(AppState::LevelSelect);
        }
        JudgmentResult::ContinueDeploy => {
            let has_remaining = deploy_res.remaining_gliders > 0 || deploy_res.remaining_lwss > 0;
            if has_remaining {
                next_state.set(AppState::Deployment);
            } else {
                dialog.0 = Some("任务失败！部署资源已耗尽。".to_string());
                next_state.set(AppState::LevelSelect);
            }
        }
        JudgmentResult::Defeat => {
            dialog.0 = Some("任务失败！部署资源已耗尽。".to_string());
            next_state.set(AppState::LevelSelect);
        }
    }
}
