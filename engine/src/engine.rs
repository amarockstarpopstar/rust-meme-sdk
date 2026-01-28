use crate::error::EngineError;
use crate::physics::PhysicsWorld;
use crate::renderer::{RenderFrame, Renderer};
use crate::scene::Scene;
use std::time::Instant;
use tracing::info;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

pub type EngineResult<T> = Result<T, EngineError>;

#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub target_fps: u32,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            title: "Meme Engine".to_string(),
            width: 1280,
            height: 720,
            target_fps: 60,
        }
    }
}

#[derive(Debug, Clone)]
pub enum EngineEvent {
    Startup,
    Frame { delta_seconds: f32 },
    Shutdown,
}

pub struct Engine {
    config: EngineConfig,
    renderer: Option<Renderer>,
    physics: PhysicsWorld,
    scene: Scene,
}

impl Engine {
    pub fn new(config: EngineConfig) -> EngineResult<Self> {
        tracing_subscriber::fmt::try_init().ok();
        let physics = PhysicsWorld::new();
        let scene = Scene::default();
        Ok(Self {
            config,
            renderer: None,
            physics,
            scene,
        })
    }

    pub fn run(mut self) -> EngineResult<()> {
        let mut event_loop = EventLoop::new().map_err(|err| {
            EngineError::WindowCreation(format!("event loop init failed: {err:?}"))
        })?;
        let window = WindowBuilder::new()
            .with_title(self.config.title.clone())
            .with_inner_size(LogicalSize::new(self.config.width, self.config.height))
            .build(&event_loop)
            .map_err(|err| EngineError::WindowCreation(err.to_string()))?;

        self.renderer = Some(Renderer::new(&window)?);
        info!("engine startup");
        let mut last_frame = Instant::now();
        let target_frame_time = 1.0 / self.config.target_fps.max(1) as f32;

        event_loop
            .run(move |event, event_loop| {
                event_loop.set_control_flow(ControlFlow::Poll);
                match event {
                    Event::WindowEvent { event, .. } => match event {
                        WindowEvent::CloseRequested => {
                            info!("engine shutdown");
                            event_loop.exit();
                        }
                        WindowEvent::Resized(size) => {
                            if let Some(renderer) = self.renderer.as_mut() {
                                renderer.resize(size.width, size.height);
                            }
                        }
                        WindowEvent::RedrawRequested => {
                            let now = Instant::now();
                            let delta = now.duration_since(last_frame).as_secs_f32();
                            if delta < target_frame_time {
                                return;
                            }
                            last_frame = now;
                            self.physics.step(delta);
                            self.scene.update(delta);
                            let frame = RenderFrame {
                                clear_color: self.scene.environment.clear_color,
                            };
                            if let Some(renderer) = self.renderer.as_mut() {
                                if let Err(err) = renderer.render(frame) {
                                    tracing::error!("render error: {err}");
                                }
                            }
                        }
                        _ => {}
                    },
                    Event::AboutToWait => {
                        window.request_redraw();
                    }
                    _ => {}
                }
            })
            .map_err(|err| {
                EngineError::WindowCreation(format!("event loop failed: {err:?}"))
            })?;
        Ok(())
    }
}
