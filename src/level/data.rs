use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::grid::{CellType, GridCoord};

/// 关卡数据（从 JSON 加载）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelData {
    pub id: String,
    pub name: String,
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default = "default_width")]
    pub width: usize,
    #[serde(default = "default_height")]
    pub height: usize,
    pub initial_cells: Vec<Vec<CellType>>,
    pub deployment_zone: Vec<GridCoord>,
    pub max_gliders: u32,
    pub max_lwss: u32,
    #[serde(default = "default_evolution_steps")]
    pub evolution_steps: u32,
}

fn default_version() -> u32 { 1 }
fn default_width() -> usize { 60 }
fn default_height() -> usize { 40 }
fn default_evolution_steps() -> u32 { 200 }

/// 进度存档
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    pub version: u32,
    pub unlocked_levels: Vec<String>,
    pub completed_levels: Vec<String>,
}

impl Default for SaveData {
    fn default() -> Self {
        Self {
            version: 1,
            unlocked_levels: vec!["level_01".to_string()],
            completed_levels: vec![],
        }
    }
}
