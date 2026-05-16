use bevy::prelude::*;

use crate::state::AppState;

/// 主菜单状态：点击"开始游戏"或"自定义"时切换状态
pub fn menu_system(
    next_state: ResMut<NextState<AppState>>,
    state: Res<State<AppState>>,
) {
    // 状态切换由 UI 触发，此 system 仅占位
    // 实际切换在 UI 系统中通过 next_state 完成
    let _ = (next_state, state);
}
