pub mod grid_renderer;
pub mod camera;
pub mod animation;
pub mod effects;

use bevy::prelude::*;

use self::animation::animation_system;
use self::camera::{setup_camera, shake_camera_system, CameraShake};
use self::effects::{deploy_effect_system, explosion_effects_system, DeployEffectEvent, ExplosionEvent};
use self::grid_renderer::{grid_render_system, ghost_preview_system, zone_render_system};

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CameraShake::default())
            .add_event::<ExplosionEvent>()
            .add_event::<DeployEffectEvent>()
            .add_systems(Startup, setup_camera)
            .add_systems(
                Update,
                (
                    grid_render_system,
                    zone_render_system,
                    ghost_preview_system,
                    shake_camera_system,
                    animation_system,
                    explosion_effects_system,
                    deploy_effect_system,
                ),
            );
    }
}
