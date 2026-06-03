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

/// 根据名称获取图案细胞（默认向右，阵营由调用方指定）
pub fn get_pattern_by_name(
    name: &str,
    origin: GridCoord,
    faction: Faction,
) -> Option<Vec<(GridCoord, CellType)>> {
    let cell = CellType::Normal(faction);
    match name {
        "Glider" => Some(glider(origin, Direction::Right, faction)),
        "LWSS" => Some(lwss(origin, Direction::Right, faction)),
        "Block" => Some(apply_pattern(BLOCK, origin, cell)),
        "Tub" => Some(apply_pattern(TUB, origin, cell)),
        "Boat" => Some(apply_pattern(BOAT, origin, cell)),
        "Loaf" => Some(apply_pattern(LOAF, origin, cell)),
        "Beehive" => Some(apply_pattern(BEEHIVE, origin, cell)),
        "Blinker" => Some(apply_pattern(BLINKER, origin, cell)),
        "Toad" => Some(apply_pattern(TOAD, origin, cell)),
        "Beacon" => Some(apply_pattern(BEACON, origin, cell)),
        "Pulsar" => Some(apply_pattern(PULSAR, origin, cell)),
        _ => None,
    }
}

fn glider(origin: GridCoord, dir: Direction, faction: Faction) -> Vec<(GridCoord, CellType)> {
    rotate_pattern(&GLIDER_RIGHT, origin, dir, CellType::Normal(faction))
}

fn lwss(origin: GridCoord, dir: Direction, faction: Faction) -> Vec<(GridCoord, CellType)> {
    rotate_pattern(&LWSS_RIGHT, origin, dir, CellType::Normal(faction))
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

/// 旋转图案（绕图案包围盒中心旋转，对应 0°/90°/180°/270°）
fn rotate_pattern(
    offsets: &[(isize, isize)],
    origin: GridCoord,
    dir: Direction,
    cell: CellType,
) -> Vec<(GridCoord, CellType)> {
    let (px, py) = pattern_pivot(offsets);

    offsets
        .iter()
        .map(|(dx, dy)| {
            let (rx, ry) = rotate_offset(dx - px, dy - py, dir);
            let x = (origin.x as isize + rx).max(0) as usize;
            let y = (origin.y as isize + ry).max(0) as usize;
            (GridCoord::new(x, y), cell)
        })
        .collect()
}

/// 图案包围盒中心（作为旋转锚点）
fn pattern_pivot(offsets: &[(isize, isize)]) -> (isize, isize) {
    let min_x = offsets.iter().map(|(x, _)| *x).min().unwrap_or(0);
    let max_x = offsets.iter().map(|(x, _)| *x).max().unwrap_or(0);
    let min_y = offsets.iter().map(|(_, y)| *y).min().unwrap_or(0);
    let max_y = offsets.iter().map(|(_, y)| *y).max().unwrap_or(0);
    ((min_x + max_x) / 2, (min_y + max_y) / 2)
}

/// 在 y 向下、x 向右的网格中顺时针旋转偏移量
fn rotate_offset(vx: isize, vy: isize, dir: Direction) -> (isize, isize) {
    match dir {
        Direction::Right => (vx, vy),
        Direction::Down => (-vy, vx),
        Direction::Left => (-vx, -vy),
        Direction::Up => (vy, -vx),
    }
}
