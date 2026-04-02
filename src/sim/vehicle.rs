// Vehicle types and per-frame physics including curved intersection turns.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Route { Right, Straight, Left }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Approach { North, South, East, West }

pub const LANE_W: f32 = 56.0;

// Lane center offsets from driver's perspective: rightmost to leftmost.
pub const LANE_OFFSETS: [f32; 3] = [
    2.5 * LANE_W,
    1.5 * LANE_W,
    0.5 * LANE_W,
];

#[derive(Clone, Debug)]
pub struct Vehicle {
    pub approach:     Approach,
    pub route:        Route,
    pub position:     (f32, f32),
    pub velocity:     f32,
    pub elapsed:      f32,
    pub heading:      (f32, f32),
    pub turned:       bool,
    pub stopped_once: bool,
    pub stop_hold:    f32,
    pub turned_sublane: Option<usize>,
}

impl Vehicle {
    pub fn new(
        approach: Approach,
        route:    Route,
        position: (f32, f32),
        velocity: f32,
    ) -> Self {
        Self {
            approach, route, position, velocity,
            elapsed:      0.0,
            heading:      heading_for_approach(approach),
            turned:       false,
            stopped_once: false,
            stop_hold:    0.0,
            turned_sublane: None,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.elapsed += dt;

        // Check if should turn
        if !self.turned && self.route != Route::Straight && self.at_turn_point() {
            self.heading = turn_heading(self.approach, self.route);
            self.turned = true;
            
            // Track which sublane this vehicle belongs in for the outgoing path
            let outgoing_sublane = match self.route {
                Route::Right    => 0,
                Route::Straight => 1,
                Route::Left     => 2,
            };
            self.turned_sublane = Some(outgoing_sublane);
        }

        // Simple forward movement
        self.position.0 += self.heading.0 * self.velocity * dt;
        self.position.1 += self.heading.1 * self.velocity * dt;
    }

    pub fn is_within_bounds(&self, limit: f32) -> bool {
        self.position.0.abs() <= limit && self.position.1.abs() <= limit
    }

    fn at_turn_point(&self) -> bool {
        let lane_offset = match self.route {
            Route::Right => 2.5 * LANE_W,
            Route::Straight => return false,
            Route::Left => 0.5 * LANE_W,
        };

        match (self.approach, self.route) {
            (Approach::North, Route::Right) => self.position.1 >= -lane_offset,
            (Approach::North, Route::Left) => self.position.1 >= lane_offset,
            (Approach::South, Route::Right) => self.position.1 <= lane_offset,
            (Approach::South, Route::Left) => self.position.1 <= -lane_offset,
            (Approach::East, Route::Right) => self.position.0 <= lane_offset,
            (Approach::East, Route::Left) => self.position.0 <= -lane_offset,
            (Approach::West, Route::Right) => self.position.0 >= -lane_offset,
            (Approach::West, Route::Left) => self.position.0 >= lane_offset,
            (_, Route::Straight) => false,
        }
    }
}

fn turn_heading(approach: Approach, route: Route) -> (f32, f32) {
    match route {
        Route::Straight => heading_for_approach(approach),
        Route::Right => match approach {
            Approach::North => (-1.0,  0.0),
            Approach::South => ( 1.0,  0.0),
            Approach::East  => ( 0.0, -1.0),
            Approach::West  => ( 0.0,  1.0),
        },
        Route::Left => match approach {
            Approach::North => ( 1.0,  0.0),
            Approach::South => (-1.0,  0.0),
            Approach::East  => ( 0.0,  1.0),
            Approach::West  => ( 0.0, -1.0),
        },
    }
}

pub fn heading_for_approach(approach: Approach) -> (f32, f32) {
    match approach {
        Approach::North => ( 0.0,  1.0),
        Approach::South => ( 0.0, -1.0),
        Approach::East  => (-1.0,  0.0),
        Approach::West  => ( 1.0,  0.0),
    }
}