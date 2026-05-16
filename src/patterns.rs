use crate::grid::{CellType, Direction, Faction, GridCoord};

/// 预设图案：一组带位置的 CellType
pub struct Pattern {
    pub name: &'static str,
    pub cells: Vec<(GridCoord, CellType)>,
}

/// 滑翔机 (Glider) - 红方，方向向右
pub fn glider_red(origin: GridCoord, dir: Direction) -> Vec<(GridCoord, CellType)> {
    rotate_pattern(&GLIDER_RIGHT, origin, dir, CellType::Normal(Faction::Red))
}

/// 滑翔机 (Glider) - 蓝方，方向向右
pub fn glider_blue(origin: GridCoord, dir: Direction) -> Vec<(GridCoord, CellType)> {
    rotate_pattern(&GLIDER_RIGHT, origin, dir, CellType::Normal(Faction::Blue))
}

/// 轻型飞船 (LWSS) - 红方，方向向右
pub fn lwss_red(origin: GridCoord, dir: Direction) -> Vec<(GridCoord, CellType)> {
    rotate_pattern(&LWSS_RIGHT, origin, dir, CellType::Normal(Faction::Red))
}

/// 轻型飞船 (LWSS) - 蓝方，方向向右
pub fn lwss_blue(origin: GridCoord, dir: Direction) -> Vec<(GridCoord, CellType)> {
    rotate_pattern(&LWSS_RIGHT, origin, dir, CellType::Normal(Faction::Blue))
}

// ---- 原始图案定义（默认朝右）----

const GLIDER_RIGHT: &[(isize, isize)] = &[
    (0, 1), (1, 2), (2, 0), (2, 1), (2, 2),
];

const LWSS_RIGHT: &[(isize, isize)] = &[
    (0, 1), (0, 2), (0, 3), (0, 4),
    (1, 0), (1, 4),
    (2, 4),
    (3, 0), (3, 3),
];

const BLOCK: &[(isize, isize)] = &[
    (0, 0), (0, 1), (1, 0), (1, 1),
];

const TUB: &[(isize, isize)] = &[
    (0, 1), (1, 0), (1, 2), (2, 1),
];

const BOAT: &[(isize, isize)] = &[
    (0, 0), (0, 1), (1, 0), (1, 2), (2, 1),
];

const LOAF: &[(isize, isize)] = &[
    (0, 1), (0, 2), (1, 0), (1, 3),
    (2, 1), (2, 3), (3, 2),
];

const BEEHIVE: &[(isize, isize)] = &[
    (0, 1), (0, 2), (1, 0), (1, 3), (2, 1), (2, 2),
];

const BLINKER: &[(isize, isize)] = &[
    (0, 1), (1, 1), (2, 1),
];

const TOAD: &[(isize, isize)] = &[
    (0, 1), (0, 2), (0, 3),
    (1, 0), (1, 1), (1, 2),
];

const BEACON: &[(isize, isize)] = &[
    (0, 0), (0, 1), (1, 0),
    (2, 3), (3, 2), (3, 3),
];

const PULSAR: &[(isize, isize)] = &[
    (1, 3), (1, 4), (1, 5),
    (3, 1), (4, 1), (5, 1),
    (3, 6), (4, 6), (5, 6),
    (6, 3), (6, 4), (6, 5),
    (8, 3), (8, 4), (8, 5),
    (3, 8), (4, 8), (5, 8),
    (3, 9), (4, 9), (5, 9),
    (11, 3), (11, 4), (11, 5),
    (11, 8), (11, 9), (11, 10),
    (10, 3), (10, 5),
    (9, 1), (10, 1), (11, 1),
    (9, 6), (10, 6), (11, 6),
    (10, 8), (10, 10),
    (3, 11), (4, 11), (5, 11),
    (8, 11), (9, 11), (10, 11),
    (1, 3), (1, 4), (1, 5),
    (6, 1), (6, 3), (6, 4), (6, 5),
    (8, 1), (8, 6),
    (13, 1), (13, 3), (13, 4), (13, 5),
    (9, 9), (10, 9), (11, 9),
];

/// 获取所有预设静物图案名称
pub fn still_life_names() -> Vec<&'static str> {
    vec!["Block", "Tub", "Boat", "Loaf", "Beehive"]
}

/// 获取所有预设振荡器图案名称
pub fn oscillator_names() -> Vec<&'static str> {
    vec!["Blinker", "Toad", "Beacon", "Pulsar"]
}

/// 获取所有可部署单位名称
pub fn deployable_names() -> Vec<&'static str> {
    vec!["Glider", "LWSS"]
}

/// 根据名称获取图案细胞（默认向右，红方）
pub fn get_pattern_by_name(name: &str, origin: GridCoord) -> Option<Vec<(GridCoord, CellType)>> {
    match name {
        "Glider" => Some(glider_red(origin, Direction::Right)),
        "LWSS" => Some(lwss_red(origin, Direction::Right)),
        "Block" => Some(apply_pattern(BLOCK, origin, CellType::Normal(Faction::Red))),
        "Tub" => Some(apply_pattern(TUB, origin, CellType::Normal(Faction::Red))),
        "Boat" => Some(apply_pattern(BOAT, origin, CellType::Normal(Faction::Red))),
        "Loaf" => Some(apply_pattern(LOAF, origin, CellType::Normal(Faction::Red))),
        "Beehive" => Some(apply_pattern(BEEHIVE, origin, CellType::Normal(Faction::Red))),
        "Blinker" => Some(apply_pattern(BLINKER, origin, CellType::Normal(Faction::Red))),
        "Toad" => Some(apply_pattern(TOAD, origin, CellType::Normal(Faction::Red))),
        "Beacon" => Some(apply_pattern(BEACON, origin, CellType::Normal(Faction::Red))),
        "Pulsar" => Some(apply_pattern(PULSAR, origin, CellType::Normal(Faction::Red))),
        _ => None,
    }
}

/// 将偏移列表应用到原点
fn apply_pattern(offsets: &[(isize, isize)], origin: GridCoord, cell: CellType) -> Vec<(GridCoord, CellType)> {
    offsets
        .iter()
        .map(|(dx, dy)| {
            let x = (origin.x as isize + dx).max(0) as usize;
            let y = (origin.y as isize + dy).max(0) as usize;
            (GridCoord::new(x, y), cell)
        })
        .collect()
}

/// 旋转图案
fn rotate_pattern(
    offsets: &[(isize, isize)],
    origin: GridCoord,
    dir: Direction,
    cell: CellType,
) -> Vec<(GridCoord, CellType)> {
    let rotated: Vec<(isize, isize)> = offsets
        .iter()
        .map(|(dx, dy)| match dir {
            Direction::Right => (*dx, *dy),
            Direction::Left => (-*dx, *dy),
            Direction::Down => (-*dy, *dx),
            Direction::Up => (*dy, -*dx),
        })
        .collect();

    // 归一化到正偏移区间
    let min_x = rotated.iter().map(|(x, _)| *x).min().unwrap_or(0);
    let min_y = rotated.iter().map(|(_, y)| *y).min().unwrap_or(0);

    rotated
        .iter()
        .map(|(dx, dy)| {
            let x = (origin.x as isize + dx - min_x).max(0) as usize;
            let y = (origin.y as isize + dy - min_y).max(0) as usize;
            (GridCoord::new(x, y), cell)
        })
        .collect()
}
