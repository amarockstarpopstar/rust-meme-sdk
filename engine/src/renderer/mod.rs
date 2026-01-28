mod dx11;

use crate::error::EngineError;
use glam::Vec4;

pub struct RenderFrame {
    pub clear_color: Vec4,
}

pub struct Renderer {
    #[cfg(target_os = "windows")]
    inner: dx11::Dx11Renderer,
}

impl Renderer {
    pub fn new(window: &winit::window::Window) -> Result<Self, EngineError> {
        #[cfg(target_os = "windows")]
        {
            let inner = dx11::Dx11Renderer::new(window)?;
            Ok(Self { inner })
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = window;
            Err(EngineError::UnsupportedPlatform(
                "DirectX 11 renderer requires Windows".to_string(),
            ))
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        #[cfg(target_os = "windows")]
        {
            self.inner.resize(width, height);
        }
    }

    pub fn render(&mut self, frame: RenderFrame) -> Result<(), EngineError> {
        #[cfg(target_os = "windows")]
        {
            self.inner.render(frame)
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = frame;
            Err(EngineError::UnsupportedPlatform(
                "DirectX 11 renderer requires Windows".to_string(),
            ))
        }
    }
}
