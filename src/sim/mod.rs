// Simulation state: lanes, vehicles, and per-frame updates.
pub mod lane;
pub mod vehicle;

use lane::Lane;
use vehicle::Approach;

pub struct Simulation {
    pub lanes: Vec<Lane>,
    pending_spawns: u32,
}

impl Simulation {
    pub fn new() -> Self {
        Self {
            lanes: vec![
                Lane::new(Approach::North),
                Lane::new(Approach::South),
                Lane::new(Approach::East),
                Lane::new(Approach::West),
            ],
            pending_spawns: 0,
        }
    }

    pub fn request_spawn(&mut self) {
        self.pending_spawns += 1;
    }

    pub fn update(&mut self) {
        if self.pending_spawns > 0 {
            self.pending_spawns = 0;
        }
    }
}
