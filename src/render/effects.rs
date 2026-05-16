use bevy::prelude::*;

use crate::render::animation::{AnimationKind, CellAnimation};
use crate::render::camera::CameraShake;
use crate::render::grid_renderer::{CELL_SIZE, HUD_OFFSET};
use crate::grid::GridCoord;

/// 爆炸事件
#[derive(Event)]
pub struct ExplosionEvent {
    pub positions: Vec<GridCoord>,
}

/// 播放爆炸特效
pub fn explosion_effects_system(
    mut commands: Commands,
    mut ev_explosion: EventReader<ExplosionEvent>,
    mut camera_shake: ResMut<CameraShake>,
    grid: Res<crate::grid::Grid>,
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
                    Sprite {
                        color: Color::srgb(1.0, 0.6, 0.0),
                        custom_size: Some(Vec2::splat(16.0)),
                        ..default()
                    },
                    Transform::from_translation(world_pos),
                    CellAnimation::new(AnimationKind::Explosion, 0.5),
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
                Sprite {
                    color: Color::srgb(1.0, 1.0, 1.0),
                    custom_size: Some(Vec2::splat(10.0)),
                    ..default()
                },
                Transform::from_translation(world_pos),
                CellAnimation::new(AnimationKind::Birth, 0.3),
            ));
        }
    }
}
