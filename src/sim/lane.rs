// Lane data structure holding vehicles by approach.
use crate::sim::vehicle::{Approach, Vehicle};

pub struct Lane {
    pub approach: Approach,
    pub vehicles: Vec<Vehicle>,
}

impl Lane {
    pub fn new(approach: Approach) -> Self {
        Self {
            approach,
            vehicles: Vec::new(),
        }
    }
}
