use bevy::prelude::*;

/// 玩家部署资源
#[derive(Resource, Clone)]
pub struct DeploymentResources {
    pub remaining_gliders: u32,
    pub remaining_lwss: u32,
    pub deployed_this_round: bool,
}

impl Default for DeploymentResources {
    fn default() -> Self {
        Self {
            remaining_gliders: 3,
            remaining_lwss: 2,
            deployed_this_round: false,
        }
    }
}
