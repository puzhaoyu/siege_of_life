use bevy::prelude::*;

/// 摄像机震动状态
#[derive(Resource, Clone)]
pub struct CameraShake {
    pub intensity: f32,    // 震动强度 (像素)
    pub duration: f32,     // 剩余持续时间 (秒)
    pub max_duration: f32,
}

impl Default for CameraShake {
    fn default() -> Self {
        Self {
            intensity: 0.0,
            duration: 0.0,
            max_duration: 0.0,
        }
    }
}

impl CameraShake {
    pub fn trigger(&mut self, intensity: f32, duration: f32) {
        self.intensity = intensity;
        self.duration = duration;
        self.max_duration = duration;
    }
}

/// 初始化摄像机
pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// 摄像机震动系统
pub fn shake_camera_system(
    time: Res<Time>,
    mut shake: ResMut<CameraShake>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
) {
    if shake.duration <= 0.0 {
        if let Ok(mut transform) = camera_query.get_single_mut() {
            transform.translation.x = 0.0;
            transform.translation.y = 0.0;
        }
        return;
    }

    shake.duration -= time.delta_secs();
    let progress = shake.duration / shake.max_duration.max(0.01);
    let current_intensity = shake.intensity * progress;

    let offset_x = (rand_offset() * current_intensity).round();
    let offset_y = (rand_offset() * current_intensity).round();

    if let Ok(mut transform) = camera_query.get_single_mut() {
        transform.translation.x = offset_x;
        transform.translation.y = offset_y;
    }
}

fn rand_offset() -> f32 {
    // 简单伪随机
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as f32
        / 1_000_000_000.0;
    (seed * 12345.6789).sin() * 2.0 - 1.0
}
