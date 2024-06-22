use std::time::Instant;

pub struct DebugState {
    last_time: Instant
}

impl DebugState {
    pub fn new() -> Self {
        Self { last_time: Instant::now() }
    }

    pub fn update(&mut self) -> anyhow::Result<()> {
        log::debug!(target: "FPS-Counter", "{:03} FPS", self.last_time.elapsed().as_secs_f64().recip());
        self.last_time = Instant::now();
        Ok(())
    }
}