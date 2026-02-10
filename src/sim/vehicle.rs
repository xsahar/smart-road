// Vehicle types and basic state used by the simulation.
#[derive(Clone, Copy, Debug)]
pub enum Route {
    Right,
    Straight,
    Left,
}

#[derive(Clone, Copy, Debug)]
pub enum Approach {
    North,
    South,
    East,
    West,
}

#[derive(Clone, Debug)]
pub struct Vehicle {
    pub approach: Approach,
    pub route: Route,
    pub position: (f32, f32),
    pub velocity: f32,
    pub elapsed: f32,
}

impl Vehicle {
    pub fn new(approach: Approach, route: Route, position: (f32, f32), velocity: f32) -> Self {
        Self {
            approach,
            route,
            position,
            velocity,
            elapsed: 0.0,
        }
    }
}
