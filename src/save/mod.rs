use bevy::prelude::*;

use crate::level::data::SaveData;
use crate::level::progression;

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        // 启动时已由 LevelPlugin 加载进度
        // 关闭时保存
        app.add_systems(Last, save_on_exit);
    }
}

fn save_on_exit(save_data: Res<SaveData>) {
    progression::save_progress(&save_data);
}
