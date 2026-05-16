use serde::{Deserialize, Serialize};

/// 两大阵营
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub enum Faction {
    #[default]
    Red,  // 玩家方
    Blue, // 敌方
}

/// 单元格内容
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum CellType {
    Empty,
    Normal(Faction),
    Wall,
    Bomb,
    HighValue,
}

impl Default for CellType {
    fn default() -> Self {
        CellType::Empty
    }
}

impl CellType {
    pub fn is_normal(&self) -> bool {
        matches!(self, CellType::Normal(_))
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, CellType::Empty)
    }

    pub fn is_wall(&self) -> bool {
        matches!(self, CellType::Wall)
    }

    pub fn is_bomb(&self) -> bool {
        matches!(self, CellType::Bomb)
    }

    pub fn is_high_value(&self) -> bool {
        matches!(self, CellType::HighValue)
    }

    /// 是否是实心格（不可被覆盖）
    pub fn is_solid(&self) -> bool {
        matches!(self, CellType::Wall | CellType::Bomb | CellType::HighValue)
    }
}

/// 网格坐标 (以 x 为列、y 为行，左上角为原点)
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct GridCoord {
    pub x: usize,
    pub y: usize,
}

impl GridCoord {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

/// 单位方向（用于旋转确认）
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// 可序列化的网格快照（用于关卡存储和撤销）
#[derive(Clone, Serialize, Deserialize)]
pub struct GridSnapshot {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Vec<CellType>>,
}
