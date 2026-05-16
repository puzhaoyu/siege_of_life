use crate::grid::{CellType, Faction, Grid, GridCoord};
use crate::gol::bomb;
use crate::gol::revival;

/// 执行一次 B3/S23 演化（双缓冲），并随后处理炸弹
/// 返回受到影响的非空坐标（用于渲染更新的 change detection）
pub fn evolution_step(grid: &mut Grid) -> (Vec<GridCoord>, bomb::BombExplosionResult) {
    let width = grid.width;
    let height = grid.height;
    let mut new_cells = vec![vec![CellType::Empty; height]; width];
    let mut changed = Vec::new();

    for x in 0..width {
        for y in 0..height {
            let coord = GridCoord::new(x, y);
            let cell = grid.cells[x][y];
            let living_count = grid.count_living_neighbors(coord);

            let new_cell = match cell {
                CellType::Normal(_) => {
                    // 存活条件：2 或 3 个活邻居
                    if living_count == 2 || living_count == 3 {
                        cell
                    } else {
                        CellType::Empty // 死亡
                    }
                }
                CellType::Empty => {
                    // 复活条件：恰好 3 个活邻居
                    if living_count == 3 {
                        let (red, blue) = grid.count_faction_neighbors(coord);
                        let faction = revival::determine_revival_faction(red, blue);
                        CellType::Normal(faction)
                    } else {
                        CellType::Empty
                    }
                }
                // Wall, Bomb, HighValue 保持不变
                other => other,
            };

            new_cells[x][y] = new_cell;

            if cell != new_cell {
                changed.push(coord);
            }
        }
    }

    // 原子替换
    grid.cells = new_cells;

    // 演化后处理炸弹
    let bomb_result = bomb::process_bombs(grid);

    (changed, bomb_result)
}

/// 高价值单位触碰检测：被红方 Normal 细胞相邻则判定摧毁
pub fn check_high_value_destruction(grid: &mut Grid) -> Vec<GridCoord> {
    let mut destroyed = Vec::new();
    let high_values = grid.find_high_values();

    for hv_coord in high_values {
        let neighbors = grid.neighbor_coords(hv_coord);
        let destroyed_by_red = neighbors.iter().any(|nc| {
            matches!(grid.get(*nc), CellType::Normal(Faction::Red))
        });

        if destroyed_by_red {
            grid.set(hv_coord, CellType::Empty);
            destroyed.push(hv_coord);
        }
    }

    destroyed
}

/// 检查所有高价值单位是否已被全部摧毁
pub fn all_high_values_destroyed(grid: &Grid) -> bool {
    grid.find_high_values().is_empty()
}
