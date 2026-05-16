use crate::grid::Faction;

/// 双阵营复活裁定算法
/// 输入：复活位置的邻居活细胞数量（红, 蓝），总数为3
/// 输出：复活后的阵营
pub fn determine_revival_faction(red_count: usize, blue_count: usize) -> Faction {
    if red_count > blue_count {
        Faction::Red
    } else {
        Faction::Blue
    }
}
