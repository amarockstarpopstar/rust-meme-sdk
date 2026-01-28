use thiserror::Error;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("platform not supported: {0}")]
    UnsupportedPlatform(String),
    #[error("window creation failed: {0}")]
    WindowCreation(String),
    #[error("renderer initialization failed: {0}")]
    RendererInit(String),
    #[error("runtime error: {0}")]
    Runtime(String),
}
