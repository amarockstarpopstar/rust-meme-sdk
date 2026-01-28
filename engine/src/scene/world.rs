use crate::scene::default_clear_color;
use glam::{Mat4, Vec3, Vec4};

#[derive(Debug, Clone)]
pub struct SceneEnvironment {
    pub clear_color: Vec4,
}

impl Default for SceneEnvironment {
    fn default() -> Self {
        Self {
            clear_color: default_clear_color(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov_y_radians: f32,
    pub aspect_ratio: f32,
    pub near: f32,
    pub far: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 2.0, -6.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            fov_y_radians: 60.0_f32.to_radians(),
            aspect_ratio: 16.0 / 9.0,
            near: 0.1,
            far: 500.0,
        }
    }
}

impl Camera {
    pub fn view_projection(&self) -> Mat4 {
        let view = Mat4::look_at_rh(self.position, self.target, self.up);
        let proj = Mat4::perspective_rh(self.fov_y_radians, self.aspect_ratio, self.near, self.far);
        proj * view
    }
}

#[derive(Debug, Clone)]
pub struct Scene {
    pub environment: SceneEnvironment,
    pub main_camera: Camera,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            environment: SceneEnvironment::default(),
            main_camera: Camera::default(),
        }
    }
}

impl Scene {
    pub fn update(&mut self, delta_seconds: f32) {
        let t = (delta_seconds * 0.2).min(1.0);
        let shift = Vec4::new(t * 0.1, 0.0, 0.0, 0.0);
        self.environment.clear_color = (self.environment.clear_color + shift).clamp(Vec4::ZERO, Vec4::ONE);
    }
}
