use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_egui::egui;

use crate::grid::{CellType, Grid};
use crate::level::data::LevelData;
use crate::state::{DeploymentZoneData, DialogMessage, EraserConfig, SimulatorDeployConfig, ZoneBrushConfig};

/// 模拟器当前关卡元数据（名称、文件路径）
#[derive(Resource, Clone)]
pub struct SimulatorLevelMeta {
    pub name: String,
    pub file_path: Option<PathBuf>,
}

/// 模拟器编辑模式相关资源（合并以降低系统参数数量）
#[derive(SystemParam)]
pub struct SimulatorEditingParams<'w> {
    pub grid: ResMut<'w, Grid>,
    pub deploy_config: ResMut<'w, SimulatorDeployConfig>,
    pub zone_data: ResMut<'w, DeploymentZoneData>,
    pub zone_brush: ResMut<'w, ZoneBrushConfig>,
    pub eraser: ResMut<'w, EraserConfig>,
    pub level_meta: ResMut<'w, SimulatorLevelMeta>,
    pub file_dialog: ResMut<'w, LevelFileDialogState>,
    pub dialog: ResMut<'w, DialogMessage>,
}

impl Default for SimulatorLevelMeta {
    fn default() -> Self {
        Self {
            name: "未命名关卡".to_string(),
            file_path: None,
        }
    }
}

type PathReceiver = Receiver<Option<PathBuf>>;

#[derive(Clone, Copy, PartialEq, Eq)]
enum NativeDialogKind {
    SaveBrowse,
    OpenBrowse,
}

/// 保存/打开对话框状态
#[derive(Resource, Default)]
pub struct LevelFileDialogState {
    pub save_modal_open: bool,
    pub draft_name: String,
    pub draft_path: String,
    pending_native: Option<(NativeDialogKind, std::sync::Mutex<PathReceiver>)>,
    pub pending_load: Option<LevelData>,
}

pub fn default_levels_dir() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_default()
        .join("levels")
}

pub fn grid_to_initial_cells(grid: &Grid) -> Vec<Vec<CellType>> {
    let width = grid.width;
    let height = grid.height;
    let mut rows = vec![vec![CellType::Empty; width]; height];
    for y in 0..height {
        for x in 0..width {
            rows[y][x] = grid.cells[x][y];
        }
    }
    rows
}

pub fn build_level_data(
    grid: &Grid,
    zone: &[crate::grid::GridCoord],
    deploy_config: &SimulatorDeployConfig,
    id: &str,
    name: &str,
) -> LevelData {
    LevelData {
        id: id.to_string(),
        name: name.to_string(),
        version: 1,
        width: grid.width,
        height: grid.height,
        initial_cells: grid_to_initial_cells(grid),
        deployment_zone: zone.to_vec(),
        max_gliders: deploy_config.max_gliders,
        max_lwss: deploy_config.max_lwss,
        evolution_steps: 200,
    }
}

pub fn id_from_name(name: &str) -> String {
    let slug: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else if c.is_whitespace() {
                '_'
            } else {
                '_'
            }
        })
        .collect();
    let trimmed = slug.trim_matches('_');
    if trimmed.is_empty() {
        "custom_level".to_string()
    } else {
        trimmed.to_string()
    }
}

pub fn id_from_path(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(id_from_name)
        .unwrap_or_else(|| "custom_level".to_string())
}

fn ensure_json_extension(path: PathBuf) -> PathBuf {
    match path.extension().and_then(|e| e.to_str()) {
        Some("json") => path,
        _ => path.with_extension("json"),
    }
}

fn default_save_filename(name: &str) -> String {
    let id = id_from_name(name);
    format!("{}.json", id)
}

fn spawn_native_dialog(
    kind: NativeDialogKind,
    start_dir: PathBuf,
    default_name: Option<String>,
) -> Receiver<Option<PathBuf>> {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut dialog = rfd::FileDialog::new()
            .add_filter("关卡 JSON", &["json"])
            .set_directory(start_dir);

        let result = match kind {
            NativeDialogKind::SaveBrowse => {
                if let Some(name) = default_name {
                    dialog = dialog.set_file_name(&name);
                }
                dialog.save_file()
            }
            NativeDialogKind::OpenBrowse => dialog.pick_file(),
        };
        let _ = tx.send(result);
    });
    rx
}

fn write_level_file(path: &Path, data: &LevelData) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(data).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())?;
    Ok(())
}

fn read_level_file(path: &Path) -> Result<LevelData, String> {
    let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| format!("JSON 解析失败: {}", e))
}

pub fn open_save_modal(dialog_state: &mut LevelFileDialogState, meta: &SimulatorLevelMeta) {
    dialog_state.draft_name = meta.name.clone();
    dialog_state.draft_path = meta
        .file_path
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| {
            default_levels_dir()
                .join(default_save_filename(&meta.name))
                .display()
                .to_string()
        });
    dialog_state.save_modal_open = true;
}

pub fn request_save_browse(dialog_state: &mut LevelFileDialogState) {
    let start_dir = path_from_draft(&dialog_state.draft_path)
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(default_levels_dir);
    let default_name = Path::new(&dialog_state.draft_path)
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .or_else(|| Some(default_save_filename(&dialog_state.draft_name)));
    let rx = spawn_native_dialog(NativeDialogKind::SaveBrowse, start_dir, default_name);
    dialog_state.pending_native = Some((
        NativeDialogKind::SaveBrowse,
        std::sync::Mutex::new(rx),
    ));
}

pub fn request_open_browse(dialog_state: &mut LevelFileDialogState, meta: &SimulatorLevelMeta) {
    let start_dir = meta
        .file_path
        .as_ref()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(default_levels_dir);
    let rx = spawn_native_dialog(NativeDialogKind::OpenBrowse, start_dir, None);
    dialog_state.pending_native = Some((
        NativeDialogKind::OpenBrowse,
        std::sync::Mutex::new(rx),
    ));
}

fn path_from_draft(draft: &str) -> PathBuf {
    PathBuf::from(draft)
}

pub fn commit_save(
    dialog_state: &LevelFileDialogState,
    grid: &Grid,
    zone: &DeploymentZoneData,
    deploy_config: &SimulatorDeployConfig,
) -> Result<(PathBuf, LevelData), String> {
    let path = ensure_json_extension(path_from_draft(&dialog_state.draft_path));
    let name = dialog_state.draft_name.trim();
    if name.is_empty() {
        return Err("请填写关卡名称".to_string());
    }
    let id = path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(id_from_name)
        .unwrap_or_else(|| id_from_name(name));
    let data = build_level_data(grid, &zone.zone, deploy_config, &id, name);
    write_level_file(&path, &data)?;
    Ok((path, data))
}

/// 处理原生文件对话框（在后台线程中弹出，主线程轮询结果）
pub fn poll_native_file_dialogs(
    mut dialog_state: ResMut<LevelFileDialogState>,
    mut meta: ResMut<SimulatorLevelMeta>,
    mut message: ResMut<DialogMessage>,
) {
    let Some((kind, rx_mutex)) = dialog_state.pending_native.take() else {
        return;
    };
    let rx = match rx_mutex.lock() {
        Ok(guard) => guard,
        Err(_) => {
            message.0 = Some("文件对话框状态异常".to_string());
            return;
        }
    };

    match rx.try_recv() {
        Ok(Some(path)) => {
            let path = ensure_json_extension(path);
            match kind {
                NativeDialogKind::SaveBrowse => {
                    dialog_state.draft_path = path.display().to_string();
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        if !stem.is_empty() {
                            dialog_state.draft_name = stem.to_string();
                        }
                    }
                }
                NativeDialogKind::OpenBrowse => match read_level_file(&path) {
                    Ok(data) => {
                        meta.name = data.name.clone();
                        meta.file_path = Some(path.clone());
                        dialog_state.pending_load = Some(data);
                        message.0 = Some(format!("已打开: {}", path.display()));
                    }
                    Err(e) => message.0 = Some(e),
                },
            }
        }
        Ok(None) => {}
        Err(std::sync::mpsc::TryRecvError::Empty) => {
            drop(rx);
            dialog_state.pending_native = Some((kind, rx_mutex));
        }
        Err(std::sync::mpsc::TryRecvError::Disconnected) => {
            message.0 = Some("文件对话框异常结束".to_string());
        }
    }
}

/// 应用待加载的关卡数据
pub fn apply_pending_level_load(
    mut dialog_state: ResMut<LevelFileDialogState>,
    mut grid: ResMut<Grid>,
    mut deploy_config: ResMut<SimulatorDeployConfig>,
    mut zone_data: ResMut<DeploymentZoneData>,
    mut meta: ResMut<SimulatorLevelMeta>,
    mut evo_config: ResMut<crate::state::EvolutionConfig>,
) {
    let Some(data) = dialog_state.pending_load.take() else {
        return;
    };
    meta.name = data.name.clone();
    crate::state::deployment::load_level_to_grid(&mut grid, &data);
    deploy_config.max_gliders = data.max_gliders;
    deploy_config.max_lwss = data.max_lwss;
    zone_data.zone = data.deployment_zone.clone();
    evo_config.steps_per_deployment = data.evolution_steps;
}

/// 保存关卡对话框（egui）
pub fn show_save_level_modal(
    ctx: &egui::Context,
    dialog_state: &mut LevelFileDialogState,
    meta: &mut SimulatorLevelMeta,
    grid: &Grid,
    zone_data: &DeploymentZoneData,
    deploy_config: &SimulatorDeployConfig,
    message: &mut DialogMessage,
) {
    if !dialog_state.save_modal_open {
        return;
    }

    let mut close = false;
    let mut do_save = false;
    let mut do_browse = false;

    egui::Window::new("保存关卡")
        .collapsible(false)
        .resizable(true)
        .default_width(420.0)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.label("关卡名称");
            ui.add(
                egui::TextEdit::singleline(&mut dialog_state.draft_name)
                    .desired_width(f32::INFINITY)
                    .hint_text("例如：我的第一关"),
            );

            ui.add_space(8.0);
            ui.label("保存位置");
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut dialog_state.draft_path)
                        .desired_width(ui.available_width() - 72.0)
                        .hint_text("路径/文件名.json"),
                );
                if ui.button("浏览…").clicked() {
                    do_browse = true;
                }
            });

            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui
                    .add_sized(
                        egui::vec2(88.0, 32.0),
                        egui::Button::new(egui::RichText::new("保存").size(14.0)),
                    )
                    .clicked()
                {
                    do_save = true;
                }
                if ui
                    .add_sized(
                        egui::vec2(88.0, 32.0),
                        egui::Button::new(egui::RichText::new("取消").size(14.0)),
                    )
                    .clicked()
                {
                    close = true;
                }
            });
        });

    if do_browse {
        request_save_browse(dialog_state);
    }

    if do_save {
        match commit_save(dialog_state, grid, zone_data, deploy_config) {
            Ok((path, data)) => {
                meta.name = data.name.clone();
                meta.file_path = Some(path.clone());
                message.0 = Some(format!("已保存: {}", path.display()));
                close = true;
            }
            Err(e) => message.0 = Some(e),
        }
    }

    if close {
        dialog_state.save_modal_open = false;
    }
}
