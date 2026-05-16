pub mod menu;
pub mod level_select;
pub mod deployment;
pub mod evolution;
pub mod judgment;
pub mod simulator;

use bevy::prelude::*;

/// 游戏主状态
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    MainMenu,
    LevelSelect,
    Deployment,
    Evolution,
    Judgment,
    Simulator,
}

/// 模拟器子状态
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SimulatorState {
    #[default]
    Editing,
    TrialPlay,
    DeploymentTest,
}

/// 判定结果
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JudgmentResult {
    Victory,
    ContinueDeploy,
    Defeat,
}

/// 演化配置资源
#[derive(Resource, Clone)]
pub struct EvolutionConfig {
    pub steps_per_deployment: u32,
    pub current_step: u32,
    pub speed_ms: f32,
    pub is_paused: bool,
    pub timer: f32,
}

impl Default for EvolutionConfig {
    fn default() -> Self {
        Self {
            steps_per_deployment: 200,
            current_step: 0,
            speed_ms: 25.0,
            is_paused: false,
            timer: 0.0,
        }
    }
}

/// 当前关卡 ID 资源
#[derive(Resource, Clone)]
pub struct CurrentLevelId(pub Option<String>);

impl Default for CurrentLevelId {
    fn default() -> Self {
        Self(None)
    }
}

/// 判定结果资源
#[derive(Resource, Clone)]
pub struct CurrentJudgment(pub Option<JudgmentResult>);

impl Default for CurrentJudgment {
    fn default() -> Self {
        Self(None)
    }
}

/// 显示对话框用的消息
#[derive(Resource, Clone)]
pub struct DialogMessage(pub Option<String>);

impl Default for DialogMessage {
    fn default() -> Self {
        Self(None)
    }
}

/// 模拟器选中的元素资源
#[derive(Resource, Clone)]
pub struct SelectedElement(pub crate::grid::CellType);

impl Default for SelectedElement {
    fn default() -> Self {
        Self(crate::grid::CellType::Normal(crate::grid::Faction::Red))
    }
}

/// 模拟器选中的预设图案名称
#[derive(Resource, Clone, Default)]
pub struct SelectedPattern(pub Option<String>);

/// 模拟器部署配置：试玩时可部署的单位数量
#[derive(Resource, Clone)]
pub struct SimulatorDeployConfig {
    pub max_gliders: u32,
    pub max_lwss: u32,
}

impl Default for SimulatorDeployConfig {
    fn default() -> Self {
        Self {
            max_gliders: 3,
            max_lwss: 2,
        }
    }
}

/// 模拟器网格快照：试玩后恢复用
#[derive(Resource, Clone, Default)]
pub struct SimulatorSnapshot {
    pub cells: Option<Vec<Vec<crate::grid::CellType>>>,
    pub zone: Option<Vec<crate::grid::GridCoord>>,
}

/// 部署区域数据
#[derive(Resource, Clone, Default)]
pub struct DeploymentZoneData {
    pub zone: Vec<crate::grid::GridCoord>,
}

/// 区域画笔配置
#[derive(Resource, Clone)]
pub struct ZoneBrushConfig {
    pub active: bool,
    pub size: u32,
}

impl Default for ZoneBrushConfig {
    fn default() -> Self {
        Self {
            active: false,
            size: 1,
        }
    }
}

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .init_state::<SimulatorState>()
            .insert_resource(EvolutionConfig::default())
            .insert_resource(CurrentLevelId::default())
            .insert_resource(CurrentJudgment::default())
            .insert_resource(DialogMessage::default())
            .insert_resource(SelectedElement::default())
            .insert_resource(SelectedPattern::default())
            .insert_resource(SimulatorDeployConfig::default())
            .insert_resource(SimulatorSnapshot::default())
            .insert_resource(DeploymentZoneData::default())
            .insert_resource(ZoneBrushConfig::default())
            // 注册状态系统
            .add_systems(OnEnter(AppState::Deployment), deployment::enter_deployment)
            .add_systems(OnEnter(AppState::Evolution), evolution::enter_evolution)
            .add_systems(OnEnter(AppState::Simulator), simulator::enter_simulator)
            .add_systems(
                Update,
                (
                    evolution::evolution_system.run_if(in_state(AppState::Evolution)),
                    judgment::judgment_system.run_if(in_state(AppState::Judgment)),
                    simulator::simulator_evolution_system.run_if(in_state(AppState::Simulator)),
                ),
            );
    }
}
