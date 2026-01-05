use std::time::Instant;

#[derive(Debug, Clone)]
pub struct Time {
    last: Instant,
}

impl Time {
    pub fn new() -> Self {
        Self {
            last: Instant::now(),
        }
    }

    pub fn tick(&mut self) -> f32 {
        let now = Instant::now();
        let dt = now.duration_since(self.last).as_secs_f32();
        self.last = now;
        dt
    }
}
