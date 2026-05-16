pub mod resources;
pub mod deploy;
pub mod palette;

use bevy::prelude::*;

use self::deploy::DragDeployState;
use self::resources::DeploymentResources;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DeploymentResources::default())
            .insert_resource(DragDeployState::default());
    }
}
