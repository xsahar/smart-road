// Vehicle types and basic state used by the simulation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Route {
    Right,
    Straight,
    Left,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    pub heading: (f32, f32),
    pub lane: u8,
    pub turned: bool,
}

impl Vehicle {
    pub fn new(approach: Approach, route: Route, lane: u8, position: (f32, f32), velocity: f32) -> Self {
        Self {
            approach,
            route,
            position,
            velocity,
            elapsed: 0.0,
            heading: heading_for_approach(approach),
            lane,
            turned: false,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.elapsed += dt;
        if self.should_turn() {
            self.heading = turn_heading(self.approach, self.route, self.heading);
            self.turned = true;
        }
        self.position.0 += self.heading.0 * self.velocity * dt;
        self.position.1 += self.heading.1 * self.velocity * dt;
    }

    pub fn is_within_bounds(&self, limit: f32) -> bool {
        self.position.0.abs() <= limit && self.position.1.abs() <= limit
    }

    fn should_turn(&self) -> bool {
        if self.route == Route::Straight || self.turned {
            return false;
        }

        const TURN_TRIGGER: f32 = 20.0;
        match self.approach {
            Approach::North => self.position.1 >= -TURN_TRIGGER,
            Approach::South => self.position.1 <= TURN_TRIGGER,
            Approach::East => self.position.0 <= TURN_TRIGGER,
            Approach::West => self.position.0 >= -TURN_TRIGGER,
        }
    }
}

fn heading_for_approach(approach: Approach) -> (f32, f32) {
    match approach {
        Approach::North => (0.0, 1.0),
        Approach::South => (0.0, -1.0),
        Approach::East => (-1.0, 0.0),
        Approach::West => (1.0, 0.0),
    }
}

fn turn_heading(approach: Approach, route: Route, current: (f32, f32)) -> (f32, f32) {
    match route {
        Route::Straight => current,
        Route::Right => match approach {
            Approach::North => (-1.0, 0.0),
            Approach::South => (1.0, 0.0),
            Approach::East => (0.0, -1.0),
            Approach::West => (0.0, 1.0),
        },
        Route::Left => match approach {
            Approach::North => (1.0, 0.0),
            Approach::South => (-1.0, 0.0),
            Approach::East => (0.0, 1.0),
            Approach::West => (0.0, -1.0),
        },
    }
}
