use bevy_egui::egui;

/// 通用按钮样式
pub fn big_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.add_sized(
        egui::vec2(200.0, 50.0),
        egui::Button::new(egui::RichText::new(text).size(24.0)),
    )
}

/// 居中标题
pub fn centered_title(ui: &mut egui::Ui, text: &str) {
    ui.vertical_centered(|ui| {
        ui.add(egui::Label::new(
            egui::RichText::new(text).size(32.0).strong(),
        ));
    });
}

/// 显示对话框
pub fn show_dialog(ctx: &egui::Context, message: &str, on_close: impl FnOnce()) {
    egui::Window::new("提示")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.label(egui::RichText::new(message).size(18.0));
            ui.add_space(16.0);
            if ui
                .add_sized(
                    egui::vec2(100.0, 36.0),
                    egui::Button::new(egui::RichText::new("确定").size(16.0)),
                )
                .clicked()
            {
                on_close();
            }
        });
}

/// 确认对话框
pub fn show_confirm_dialog(
    ctx: &egui::Context,
    title: &str,
    message: &str,
    on_confirm: impl FnOnce(),
    on_cancel: impl FnOnce(),
) {
    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.label(egui::RichText::new(message).size(16.0));
            ui.add_space(16.0);
            ui.horizontal(|ui| {
                if ui
                    .add_sized(
                        egui::vec2(80.0, 32.0),
                        egui::Button::new(egui::RichText::new("确定").size(14.0)),
                    )
                    .clicked()
                {
                    on_confirm();
                }
                if ui
                    .add_sized(
                        egui::vec2(80.0, 32.0),
                        egui::Button::new(egui::RichText::new("取消").size(14.0)),
                    )
                    .clicked()
                {
                    on_cancel();
                }
            });
        });
}
