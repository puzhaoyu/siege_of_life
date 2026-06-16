use bevy::prelude::*;
use bevy::sprite::TextureAtlasLayout;

use crate::render::grid_renderer::CELL_SIZE;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AnimationKind {
    Explosion,
    TreasureGlow,
    Clash,
    Birth,
}

#[derive(Component)]
pub struct FrameAnimation {
    pub kind: AnimationKind,
    pub frame_count: usize,
    pub current_frame: usize,
    pub frame_timer: Timer,
    pub despawn_on_finish: bool,
    pub looped: bool,
}

impl FrameAnimation {
    pub fn one_shot(kind: AnimationKind, frame_count: usize, frame_duration: f32) -> Self {
        Self {
            kind,
            frame_count,
            current_frame: 0,
            frame_timer: Timer::from_seconds(frame_duration, TimerMode::Repeating),
            despawn_on_finish: true,
            looped: false,
        }
    }
}

#[derive(Resource, Clone)]
pub struct EffectAnimationAssets {
    pub explosion_image: Handle<Image>,
    pub treasure_image: Handle<Image>,
    pub clash_image: Handle<Image>,
    pub explosion_layout: Handle<TextureAtlasLayout>,
    pub treasure_layout: Handle<TextureAtlasLayout>,
    pub clash_layout: Handle<TextureAtlasLayout>,
}

impl EffectAnimationAssets {
    pub fn image_for(&self, kind: AnimationKind) -> Handle<Image> {
        match kind {
            AnimationKind::Explosion => self.explosion_image.clone(),
            AnimationKind::TreasureGlow => self.treasure_image.clone(),
            AnimationKind::Clash => self.clash_image.clone(),
            AnimationKind::Birth => self.treasure_image.clone(),
        }
    }

    pub fn layout_for(&self, kind: AnimationKind) -> Handle<TextureAtlasLayout> {
        match kind {
            AnimationKind::Explosion => self.explosion_layout.clone(),
            AnimationKind::TreasureGlow => self.treasure_layout.clone(),
            AnimationKind::Clash => self.clash_layout.clone(),
            AnimationKind::Birth => self.treasure_layout.clone(),
        }
    }
}

pub fn setup_effect_animation_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let frame_size = UVec2::new(362, 724);
    let columns = 6;
    let rows = 1;

    let assets = EffectAnimationAssets {
        explosion_image: asset_server.load("effects/explosion_sheet.png"),
        treasure_image: asset_server.load("effects/treasure_sheet.png"),
        clash_image: asset_server.load("effects/clash_sheet.png"),
        explosion_layout: atlas_layouts.add(TextureAtlasLayout::from_grid(frame_size, columns, rows, None, None)),
        treasure_layout: atlas_layouts.add(TextureAtlasLayout::from_grid(frame_size, columns, rows, None, None)),
        clash_layout: atlas_layouts.add(TextureAtlasLayout::from_grid(frame_size, columns, rows, None, None)),
    };

    commands.insert_resource(assets);
}

pub fn animation_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut FrameAnimation, &mut Sprite, &mut Transform)>,
) {
    for (entity, mut animation, mut sprite, mut transform) in &mut query {
        animation.frame_timer.tick(time.delta());
        if !animation.frame_timer.just_finished() {
            continue;
        }

        let next_frame = animation.current_frame + 1;
        if next_frame >= animation.frame_count {
            if animation.looped {
                animation.current_frame = 0;
            } else if animation.despawn_on_finish {
                commands.entity(entity).despawn();
                continue;
            }
        } else {
            animation.current_frame = next_frame;
        }

        if let Some(atlas) = sprite.texture_atlas.as_mut() {
            atlas.index = animation.current_frame;
        }

        if matches!(animation.kind, AnimationKind::TreasureGlow | AnimationKind::Birth) {
            let pulse = 1.0 + (animation.current_frame as f32 / animation.frame_count as f32) * 0.15;
            let base = transform.scale.x.max(CELL_SIZE / 362.0);
            transform.scale = Vec3::splat(base * pulse);
        }
    }
}
