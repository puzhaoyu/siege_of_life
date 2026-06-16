use std::collections::HashSet;

use crate::gol::bomb;
use crate::gol::revival;
use crate::grid::{CellType, Faction, Grid, GridCoord};

pub struct EvolutionStepResult {
    pub changed: Vec<GridCoord>,
    pub bomb_result: bomb::BombExplosionResult,
    pub clash_positions: Vec<GridCoord>,
}

/// 执行一次 B3/S23 演化（双缓冲），并随后处理炸弹
pub fn evolution_step(grid: &mut Grid) -> EvolutionStepResult {
    let width = grid.width;
    let height = grid.height;
    let before_cells = grid.cells.clone();
    let mut new_cells = vec![vec![CellType::Empty; height]; width];
    let mut changed = Vec::new();
    let mut clash_positions = HashSet::new();

    for x in 0..width {
        for y in 0..height {
            let coord = GridCoord::new(x, y);
            let cell = grid.cells[x][y];
            let living_count = grid.count_living_neighbors(coord);

            let new_cell = match cell {
                CellType::Normal(_) => {
                    if living_count == 2 || living_count == 3 {
                        cell
                    } else {
                        let (red, blue) = grid.count_faction_neighbors(coord);
                        if red > 0 && blue > 0 {
                            clash_positions.insert(coord);
                        }
                        CellType::Empty
                    }
                }
                CellType::Empty => {
                    if living_count == 3 {
                        let (red, blue) = grid.count_faction_neighbors(coord);
                        let faction = revival::determine_revival_faction(red, blue);
                        CellType::Normal(faction)
                    } else {
                        CellType::Empty
                    }
                }
                other => other,
            };

            new_cells[x][y] = new_cell;
            if cell != new_cell {
                changed.push(coord);
            }
        }
    }

    grid.cells = new_cells;
    let bomb_result = bomb::process_bombs(grid);

    for coord in &bomb_result.affected_cells {
        if matches!(before_cells[coord.x][coord.y], CellType::Normal(_)) {
            let neighbors = grid.neighbor_coords(*coord);
            let had_red = neighbors
                .iter()
                .any(|nc| matches!(before_cells[nc.x][nc.y], CellType::Normal(Faction::Red)));
            let had_blue = neighbors
                .iter()
                .any(|nc| matches!(before_cells[nc.x][nc.y], CellType::Normal(Faction::Blue)));
            if had_red && had_blue {
                clash_positions.insert(*coord);
            }
        }
    }

    EvolutionStepResult {
        changed,
        bomb_result,
        clash_positions: clash_positions.into_iter().collect(),
    }
}

/// 高价值单位触碰检测：被红方 Normal 细胞相邻则判定摧毁
pub fn check_high_value_destruction(grid: &mut Grid) -> Vec<GridCoord> {
    let mut destroyed = Vec::new();
    let high_values = grid.find_high_values();

    for hv_coord in high_values {
        let neighbors = grid.neighbor_coords(hv_coord);
        let destroyed_by_red = neighbors
            .iter()
            .any(|nc| matches!(grid.get(*nc), CellType::Normal(Faction::Red)));

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
