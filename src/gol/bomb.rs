use std::collections::{HashSet, VecDeque};

use crate::grid::{CellType, Grid, GridCoord};

/// 炸弹触发事件数据
pub struct BombExplosionResult {
    /// 所有被爆炸影响的坐标集合
    pub affected_cells: HashSet<GridCoord>,
    /// 实际被触发的炸弹坐标（用于爆炸特效）
    pub triggered_bombs: Vec<GridCoord>,
}

/// 处理炸弹：扫描所有炸弹，检测是否被 Normal 细胞触碰，执行链式引爆
pub fn process_bombs(grid: &mut Grid) -> BombExplosionResult {
    let mut affected = HashSet::new();

    // 1. 找出所有因相邻 Normal 细胞而触发的炸弹
    let triggered_bombs: Vec<GridCoord> = grid
        .find_bombs()
        .iter()
        .filter(|c| grid.has_normal_neighbor(**c))
        .cloned()
        .collect();

    if triggered_bombs.is_empty() {
        return BombExplosionResult {
            affected_cells: affected,
            triggered_bombs: vec![],
        };
    }

    // 2. BFS 链式引爆
    let mut queue: VecDeque<GridCoord> = triggered_bombs.into();
    let mut visited: HashSet<GridCoord> = HashSet::new();

    while let Some(bomb_pos) = queue.pop_front() {
        if !visited.insert(bomb_pos) {
            continue;
        }

        // 获取半径 5 的圆形爆炸范围
        let blast_zone = grid.cells_in_radius(bomb_pos, 5);
        for coord in blast_zone {
            affected.insert(coord);

            // 如果范围内有其他炸弹且未被访问过，加入队列（链式引爆）
            if grid.get(coord).is_bomb() && !visited.contains(&coord) {
                queue.push_back(coord);
            }
        }
    }

    // 3. 清除所有受影响的位置
    for coord in &affected {
        grid.set(*coord, CellType::Empty);
    }

    BombExplosionResult {
        affected_cells: affected,
        triggered_bombs: visited.into_iter().collect(),
    }
}
