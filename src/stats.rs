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
}
