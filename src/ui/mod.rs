pub mod common;
pub mod main_menu;
pub mod level_select_ui;
pub mod hud;
pub mod simulator_panel;
pub mod deployment_panel;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use self::hud::hud_ui;
use self::level_select_ui::level_select_ui;
use self::main_menu::main_menu_ui;
use self::simulator_panel::simulator_panel_ui;
use self::deployment_panel::deployment_panel_ui;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_chinese_font)
            .add_systems(
                Update,
                (
                    main_menu_ui,
                    level_select_ui,
                    hud_ui,
                    simulator_panel_ui,
                    deployment_panel_ui,
                ),
            );
    }
}

/// 配置中文字体，解决中文显示为方框的问题
fn setup_chinese_font(mut contexts: EguiContexts) {
    let ctx = contexts.ctx_mut();

    // 尝试加载系统黑体字体（macOS 自带）
    let font_paths = [
        "/System/Library/Fonts/STHeiti Medium.ttc",
        "/System/Library/Fonts/STHeiti Light.ttc",
        "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
    ];

    for path in font_paths {
        if let Ok(font_data) = std::fs::read(path) {
            let mut fonts = egui::FontDefinitions::default();

            fonts.font_data.insert(
                "chinese".to_owned(),
                egui::FontData::from_owned(font_data).tweak(
                    egui::FontTweak {
                        scale: 1.0,
                        ..Default::default()
                    },
                ),
            );

            // 将中文字体设置为首选 Proportional 字体
            fonts
                .families
                .get_mut(&egui::FontFamily::Proportional)
                .unwrap()
                .insert(0, "chinese".to_owned());

            // 等宽字体也加入作为后备
            fonts
                .families
                .get_mut(&egui::FontFamily::Monospace)
                .unwrap()
                .push("chinese".to_owned());

            ctx.set_fonts(fonts);
            return;
        }
    }

    // 如果系统字体都找不到，尝试 PingFang
    if let Ok(font_data) = std::fs::read("/System/Library/Fonts/PingFang.ttc") {
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "chinese".to_owned(),
            egui::FontData::from_owned(font_data),
        );
        fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .insert(0, "chinese".to_owned());
        ctx.set_fonts(fonts);
    }
}
