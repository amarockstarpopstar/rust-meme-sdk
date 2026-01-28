use meme_engine::{Engine, EngineConfig};

fn main() {
    let config = EngineConfig {
        title: "Meme Engine Demo".to_string(),
        width: 1280,
        height: 720,
        target_fps: 60,
    };

    let engine = match Engine::new(config) {
        Ok(engine) => engine,
        Err(err) => {
            eprintln!("engine init failed: {err}");
            return;
        }
    };

    if let Err(err) = engine.run() {
        eprintln!("engine runtime error: {err}");
    }
}
