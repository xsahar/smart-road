// Lane data structure holding vehicles by approach.
use crate::sim::vehicle::{Approach, Vehicle};

// Each `Lane` represents an approach (North/South/East/West) and contains
// three incoming lane queues (right, straight, left). Vehicles spawn into
// the appropriate incoming queue and follow that lane's path.
pub struct Lane {
    pub approach: Approach,
    // incoming[0] = Right, [1] = Straight, [2] = Left
    pub incoming: [Vec<Vehicle>; 3],
}

impl Lane {
    pub fn new(approach: Approach) -> Self {
        Self {
            approach,
            incoming: [Vec::new(), Vec::new(), Vec::new()],
        }
    }
}
