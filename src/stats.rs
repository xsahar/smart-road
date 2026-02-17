// Statistics tracked across the simulation run.
#[derive(Default)]
pub struct Stats {
    pub max_passed: u32,
    pub max_velocity: f32,
    pub min_velocity: f32,
    pub max_time: f32,
    pub min_time: f32,
    pub close_calls: u32,
}

impl Stats {
    pub fn new() -> Self {
        Self {
            min_velocity: f32::INFINITY,
            min_time: f32::INFINITY,
            ..Default::default()
        }
    }

    pub fn record_spawn(&mut self, velocity: f32) {
        if velocity > self.max_velocity {
            self.max_velocity = velocity;
        }
        if velocity < self.min_velocity {
            self.min_velocity = velocity;
        }
    }

    pub fn record_exit(&mut self, elapsed: f32) {
        self.max_passed += 1;
        if elapsed > self.max_time {
            self.max_time = elapsed;
        }
        if elapsed < self.min_time {
            self.min_time = elapsed;
        }
    }

    pub fn record_close_call(&mut self) {
        self.close_calls += 1;
    }

    pub fn summary_line(&self) -> String {
        let min_velocity = if self.min_velocity.is_finite() {
            self.min_velocity
        } else {
            0.0
        };
        let min_time = if self.min_time.is_finite() { self.min_time } else { 0.0 };
        format!(
            "smart-road | passed: {} | v(min/max): {:.1}/{:.1} | t(min/max): {:.1}/{:.1} | close: {}",
            self.max_passed,
            min_velocity,
            self.max_velocity,
            min_time,
            self.max_time,
            self.close_calls
        )
    }
}
