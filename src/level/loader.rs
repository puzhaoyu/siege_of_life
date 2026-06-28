use std::collections::HashMap;

use bevy::prelude::*;
use crate::level::data::LevelData;

/// 关卡注册表：存储所有已加载的关卡
#[derive(Resource, Default)]
pub struct LevelRegistry {
    pub levels: HashMap<String, LevelData>,
    pub ordered_ids: Vec<String>,
}

impl LevelRegistry {
    pub fn register(&mut self, data: LevelData) {
        self.ordered_ids.push(data.id.clone());
        self.levels.insert(data.id.clone(), data);
    }

    pub fn get(&self, id: &str) -> Option<&LevelData> {
        self.levels.get(id)
    }

    pub fn get_by_index(&self, index: usize) -> Option<&LevelData> {
        self.ordered_ids.get(index).and_then(|id| self.levels.get(id))
    }

    pub fn level_count(&self) -> usize {
        self.ordered_ids.len()
    }
}

/// 从内置 JSON 字符串加载所有关卡
pub fn load_builtin_levels() -> LevelRegistry {
    let mut registry = LevelRegistry::default();

    // 编译期嵌入所有关卡 JSON
    let files: &[(&str, &str)] = &[
        ("level_01", include_str!("../../assets/levels/level_01.json")),
        ("level_02", include_str!("../../assets/levels/level_02.json")),
        ("level_03", include_str!("../../assets/levels/level_03.json")),
        ("level_04", include_str!("../../assets/levels/level_04.json")),
        ("level_05", include_str!("../../assets/levels/level_05.json")),
    ];

    for (label, data_str) in files {
        match serde_json::from_str::<LevelData>(data_str) {
            Ok(data) => {
                println!("[Loader] 关卡 '{}' ({}) 加载成功", data.name, data.id);
                registry.register(data);
            }
            Err(e) => {
                println!("[Loader] 关卡 '{}' 解析失败: {}", label, e);
            }
        }
    }

    println!("[Loader] 内置关卡加载完成，共 {} 个关卡", registry.level_count());
    registry
}
