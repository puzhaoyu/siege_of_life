use bevy::prelude::*;

/// 动画状态组件
#[derive(Component)]
pub struct CellAnimation {
    pub kind: AnimationKind,
    pub progress: f32,
    pub duration: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AnimationKind {
    Birth,
    Death,
    Explosion,
}

impl CellAnimation {
    pub fn new(kind: AnimationKind, duration: f32) -> Self {
        Self {
            kind,
            progress: 0.0,
            duration,
        }
    }
}

/// 细胞动画系统：更新缩放与透明度
pub fn animation_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut CellAnimation, &mut Transform, &mut Sprite)>,
) {
    for (entity, mut anim, mut transform, mut sprite) in query.iter_mut() {
        anim.progress += time.delta_secs();

        if anim.progress >= anim.duration {
            // 动画结束
            match anim.kind {
                AnimationKind::Birth => {
                    transform.scale = Vec3::ONE;
                    sprite.color.set_alpha(1.0);
                }
                AnimationKind::Death | AnimationKind::Explosion => {
                    commands.entity(entity).despawn();
                }
            }
            continue;
        }

        let t = anim.progress / anim.duration;

        match anim.kind {
            AnimationKind::Birth => {
                let scale = ease_out_back(t);
                transform.scale = Vec3::splat(scale);
                sprite.color.set_alpha(t);
            }
            AnimationKind::Death => {
                let scale = 1.0 - ease_in_quad(t);
                transform.scale = Vec3::splat(scale);
                sprite.color.set_alpha(1.0 - t);
            }
            AnimationKind::Explosion => {
                let scale = 1.0 + t * 2.0;
                transform.scale = Vec3::splat(scale);
                sprite.color.set_alpha(1.0 - t);
            }
        }
    }
}

fn ease_out_back(t: f32) -> f32 {
    let c1 = 1.70158;
    let c3 = c1 + 1.0;
    1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
}

fn ease_in_quad(t: f32) -> f32 {
    t * t
}
