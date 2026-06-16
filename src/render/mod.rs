pub mod grid_renderer;
pub mod camera;
pub mod animation;
pub mod effects;

use bevy::prelude::*;

use self::animation::{animation_system, setup_effect_animation_assets};
use self::camera::{setup_camera, shake_camera_system, CameraShake};
use self::effects::{
    clash_effect_system, deploy_effect_system, explosion_effects_system,
    treasure_glow_effect_system, ClashEffectEvent, DeployEffectEvent, ExplosionEvent,
    TreasureGlowEvent,
};
use self::grid_renderer::{
    ghost_preview_system, grid_lines_render_system, grid_render_system, load_textures,
    zone_render_system,
};

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CameraShake::default())
            .add_event::<ExplosionEvent>()
            .add_event::<TreasureGlowEvent>()
            .add_event::<ClashEffectEvent>()
            .add_event::<DeployEffectEvent>()
            .add_systems(
                Startup,
                (load_textures, setup_camera, setup_effect_animation_assets).chain(),
            )
            .add_systems(
                Update,
                (
                    grid_lines_render_system,
                    grid_render_system,
                    zone_render_system,
                    ghost_preview_system,
                    shake_camera_system,
                    animation_system,
                    explosion_effects_system,
                    treasure_glow_effect_system,
                    clash_effect_system,
                    deploy_effect_system,
                ),
            );
    }
}
