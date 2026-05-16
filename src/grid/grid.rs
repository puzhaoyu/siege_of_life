use bevy::prelude::*;
use crate::grid::types::{CellType, Faction, GridCoord, GridSnapshot};

/// 60×40 网格 (x 横向, y 纵向)
#[derive(Resource)]
pub struct Grid {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Vec<CellType>>, // cells[x][y]
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        let cells = vec![vec![CellType::Empty; height]; width];
        Self { width, height, cells }
    }

    pub fn get(&self, coord: GridCoord) -> CellType {
        if !self.in_bounds(coord) {
            return CellType::Wall; // 边界外视为墙
        }
        self.cells[coord.x][coord.y]
    }

    pub fn set(&mut self, coord: GridCoord, cell: CellType) {
        if self.in_bounds(coord) {
            self.cells[coord.x][coord.y] = cell;
        }
    }

    pub fn in_bounds(&self, coord: GridCoord) -> bool {
        coord.x < self.width && coord.y < self.height
    }

    pub fn is_empty(&self, coord: GridCoord) -> bool {
        self.in_bounds(coord) && self.cells[coord.x][coord.y].is_empty()
    }

    /// 获取 8 邻域 (Moore neighborhood) 坐标列表
    pub fn neighbor_coords(&self, coord: GridCoord) -> Vec<GridCoord> {
        let mut result = Vec::with_capacity(8);
        let cx = coord.x as isize;
        let cy = coord.y as isize;
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = cx + dx;
                let ny = cy + dy;
                if nx >= 0 && ny >= 0 {
                    let coord = GridCoord::new(nx as usize, ny as usize);
                    if self.in_bounds(coord) {
                        result.push(coord);
                    }
                }
            }
        }
        result
    }

    /// 统计 8 邻域中 Normal 细胞的数量（任意阵营）
    pub fn count_living_neighbors(&self, coord: GridCoord) -> usize {
        self.neighbor_coords(coord)
            .iter()
            .filter(|c| self.cells[c.x][c.y].is_normal())
            .count()
    }

    /// 统计 8 邻域中每种阵营的 Normal 细胞数量
    pub fn count_faction_neighbors(&self, coord: GridCoord) -> (usize, usize) {
        let mut red = 0;
        let mut blue = 0;
        for nc in self.neighbor_coords(coord) {
            if let CellType::Normal(f) = self.cells[nc.x][nc.y] {
                match f {
                    Faction::Red => red += 1,
                    Faction::Blue => blue += 1,
                }
            }
        }
        (red, blue)
    }

    /// 获取指定坐标周围圆形范围 (Chebyshev 距离 <= radius)
    pub fn cells_in_radius(&self, center: GridCoord, radius: usize) -> Vec<GridCoord> {
        let mut result = Vec::new();
        let r = radius as isize;
        let cx = center.x as isize;
        let cy = center.y as isize;
        for dy in -r..=r {
            for dx in -r..=r {
                if dx.abs().max(dy.abs()) <= r {
                    let nx = cx + dx;
                    let ny = cy + dy;
                    if nx >= 0 && ny >= 0 {
                        let coord = GridCoord::new(nx as usize, ny as usize);
                        if self.in_bounds(coord) {
                            result.push(coord);
                        }
                    }
                }
            }
        }
        result
    }

    /// 统计网格中所有高价值单位的位置
    pub fn find_high_values(&self) -> Vec<GridCoord> {
        let mut result = Vec::new();
        for x in 0..self.width {
            for y in 0..self.height {
                if self.cells[x][y].is_high_value() {
                    result.push(GridCoord::new(x, y));
                }
            }
        }
        result
    }

    /// 统计网格中所有炸弹的位置
    pub fn find_bombs(&self) -> Vec<GridCoord> {
        let mut result = Vec::new();
        for x in 0..self.width {
            for y in 0..self.height {
                if self.cells[x][y].is_bomb() {
                    result.push(GridCoord::new(x, y));
                }
            }
        }
        result
    }

    /// 检查指定坐标是否与任何 Normal 细胞相邻
    pub fn has_normal_neighbor(&self, coord: GridCoord) -> bool {
        self.neighbor_coords(coord)
            .iter()
            .any(|c| self.cells[c.x][c.y].is_normal())
    }

    /// 创建网格快照（用于撤销或序列化）
    pub fn snapshot(&self) -> GridSnapshot {
        GridSnapshot {
            width: self.width,
            height: self.height,
            cells: self.cells.clone(),
        }
    }

    /// 从快照恢复
    pub fn restore(&mut self, snapshot: &GridSnapshot) {
        self.width = snapshot.width;
        self.height = snapshot.height;
        self.cells = snapshot.cells.clone();
    }

    /// 清空整个网格
    pub fn clear(&mut self) {
        for x in 0..self.width {
            for y in 0..self.height {
                self.cells[x][y] = CellType::Empty;
            }
        }
    }

    /// 从坐标列表中构建部署区域
    pub fn is_in_deployment_zone(&self, coord: GridCoord, zone: &[GridCoord]) -> bool {
        zone.contains(&coord)
    }
}

impl Clone for Grid {
    fn clone(&self) -> Self {
        Self {
            width: self.width,
            height: self.height,
            cells: self.cells.clone(),
        }
    }
}
