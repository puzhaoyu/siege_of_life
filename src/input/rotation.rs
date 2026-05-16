use bevy::prelude::*;

use crate::player::deploy::DeployPhase;

/// 旋转确认交互：根据鼠标移动方向推断旋转方向
pub fn rotation_system(
    drag_state: Res<crate::player::deploy::DragDeployState>,
    windows: Query<&Window>,
) {
    if drag_state.phase != DeployPhase::Rotating {
        return;
    }
    // 旋转方向已由 drag_drop_system 计算
    let _ = windows;
}
