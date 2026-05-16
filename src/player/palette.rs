/// 可选单位面板数据
pub struct PaletteItem {
    pub name: &'static str,
    pub unit_type: super::deploy::DeployUnitType,
}

pub fn get_palette_items() -> Vec<PaletteItem> {
    vec![
        PaletteItem {
            name: "滑翔机",
            unit_type: super::deploy::DeployUnitType::Glider,
        },
        PaletteItem {
            name: "LWSS",
            unit_type: super::deploy::DeployUnitType::LWSS,
        },
    ]
}
