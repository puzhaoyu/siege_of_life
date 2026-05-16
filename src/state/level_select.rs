use bevy::prelude::*;

use crate::state::AppState;

/// 关卡选择状态：初始化占位
pub fn level_select_system(
    next_state: ResMut<NextState<AppState>>,
    state: Res<State<AppState>>,
) {
    let _ = (next_state, state);
}
