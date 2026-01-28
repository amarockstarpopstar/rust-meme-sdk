mod world;

use glam::Vec4;

pub use world::{Camera, Scene, SceneEnvironment};

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub position: glam::Vec3,
    pub rotation: glam::Quat,
    pub scale: glam::Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: glam::Vec3::ZERO,
            rotation: glam::Quat::IDENTITY,
            scale: glam::Vec3::ONE,
        }
    }
}

pub fn default_clear_color() -> Vec4 {
    Vec4::new(0.08, 0.09, 0.14, 1.0)
}
