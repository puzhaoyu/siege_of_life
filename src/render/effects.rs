use bevy::prelude::*;

use crate::grid::GridCoord;
use crate::render::animation::{AnimationKind, EffectAnimationAssets, FrameAnimation};
use crate::render::camera::CameraShake;
use crate::render::grid_renderer::{CELL_SIZE, HUD_OFFSET};

/// 爆炸事件
#[derive(Event)]
pub struct ExplosionEvent {
    pub positions: Vec<GridCoord>,
}

#[derive(Event)]
pub struct TreasureGlowEvent {
    pub positions: Vec<GridCoord>,
}

#[derive(Event)]
pub struct ClashEffectEvent {
    pub positions: Vec<GridCoord>,
}

/// 播放爆炸特效
pub fn explosion_effects_system(
    mut commands: Commands,
    mut ev_explosion: EventReader<ExplosionEvent>,
    mut camera_shake: ResMut<CameraShake>,
    grid: Res<crate::grid::Grid>,
    effect_assets: Res<EffectAnimationAssets>,
) {
    for event in ev_explosion.read() {
        if !event.positions.is_empty() {
            camera_shake.trigger(8.0, 0.5);

            let grid_w = grid.width as f32;
            let grid_h = grid.height as f32;
            let offset_x = -grid_w * CELL_SIZE / 2.0;
            let offset_y = grid_h * CELL_SIZE / 2.0 - HUD_OFFSET;

            for pos in &event.positions {
                let world_pos = Vec3::new(
                    offset_x + pos.x as f32 * CELL_SIZE + CELL_SIZE / 2.0,
                    offset_y - pos.y as f32 * CELL_SIZE - CELL_SIZE / 2.0,
                    0.1,
                );

                commands.spawn((
                    Sprite::from_atlas_image(
                        effect_assets.image_for(AnimationKind::Explosion),
                        TextureAtlas {
                            layout: effect_assets.layout_for(AnimationKind::Explosion),
                            index: 0,
                        },
                    ),
                    Transform::from_translation(world_pos)
                        .with_scale(Vec3::splat((CELL_SIZE / 362.0) * 1.05)),
                    FrameAnimation::one_shot(AnimationKind::Explosion, 6, 0.05),
                ));
            }
        }
    }
}

/// 部署放置事件
#[derive(Event)]
pub struct DeployEffectEvent {
    pub positions: Vec<GridCoord>,
}

/// 部署放置特效
pub fn deploy_effect_system(
    mut commands: Commands,
    mut ev_deploy: EventReader<DeployEffectEvent>,
    grid: Res<crate::grid::Grid>,
    effect_assets: Res<EffectAnimationAssets>,
) {
    for event in ev_deploy.read() {
        let grid_w = grid.width as f32;
        let grid_h = grid.height as f32;
        let offset_x = -grid_w * CELL_SIZE / 2.0;
        let offset_y = grid_h * CELL_SIZE / 2.0 - HUD_OFFSET;

        for pos in &event.positions {
            let world_pos = Vec3::new(
                offset_x + pos.x as f32 * CELL_SIZE + CELL_SIZE / 2.0,
                offset_y - pos.y as f32 * CELL_SIZE - CELL_SIZE / 2.0,
                0.1,
            );

            commands.spawn((
                Sprite::from_atlas_image(
                    effect_assets.image_for(AnimationKind::Birth),
                    TextureAtlas {
                        layout: effect_assets.layout_for(AnimationKind::Birth),
                        index: 0,
                    },
                ),
                Transform::from_translation(world_pos).with_scale(Vec3::splat(CELL_SIZE / 362.0)),
                FrameAnimation::one_shot(AnimationKind::Birth, 6, 0.04),
            ));
        }
    }
}

pub fn treasure_glow_effect_system(
    mut commands: Commands,
    mut ev_glow: EventReader<TreasureGlowEvent>,
    grid: Res<crate::grid::Grid>,
    effect_assets: Res<EffectAnimationAssets>,
) {
    spawn_effects(
        &mut commands,
        &mut ev_glow,
        &grid,
        &effect_assets,
        AnimationKind::TreasureGlow,
        0.06,
        0.11,
        1.45,
    );
}

pub fn clash_effect_system(
    mut commands: Commands,
    mut ev_clash: EventReader<ClashEffectEvent>,
    grid: Res<crate::grid::Grid>,
    effect_assets: Res<EffectAnimationAssets>,
) {
    spawn_effects(
        &mut commands,
        &mut ev_clash,
        &grid,
        &effect_assets,
        AnimationKind::Clash,
        0.05,
        0.12,
        1.0,
    );
}

fn spawn_effects<T: EffectPositionsEvent>(
    commands: &mut Commands,
    events: &mut EventReader<T>,
    grid: &crate::grid::Grid,
    effect_assets: &EffectAnimationAssets,
    kind: AnimationKind,
    frame_duration: f32,
    z: f32,
    base_scale: f32,
) {
    let grid_w = grid.width as f32;
    let grid_h = grid.height as f32;
    let offset_x = -grid_w * CELL_SIZE / 2.0;
    let offset_y = grid_h * CELL_SIZE / 2.0 - HUD_OFFSET;

    for event in events.read() {
        for pos in event.positions() {
            let world_pos = Vec3::new(
                offset_x + pos.x as f32 * CELL_SIZE + CELL_SIZE / 2.0,
                offset_y - pos.y as f32 * CELL_SIZE - CELL_SIZE / 2.0,
                z,
            );

            commands.spawn((
                Sprite::from_atlas_image(
                    effect_assets.image_for(kind),
                    TextureAtlas {
                        layout: effect_assets.layout_for(kind),
                        index: 0,
                    },
                ),
                Transform::from_translation(world_pos)
                    .with_scale(Vec3::splat((CELL_SIZE / 362.0) * base_scale)),
                FrameAnimation::one_shot(kind, 6, frame_duration),
            ));
        }
    }
}

pub trait EffectPositionsEvent: Event {
    fn positions(&self) -> &[GridCoord];
}

impl EffectPositionsEvent for TreasureGlowEvent {
    fn positions(&self) -> &[GridCoord] {
        &self.positions
    }
}

impl EffectPositionsEvent for ClashEffectEvent {
    fn positions(&self) -> &[GridCoord] {
        &self.positions
    }
}
