// Vehicle types and per-frame physics including curved intersection turns.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Route { Right, Straight, Left }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Approach { North, South, East, West }


pub const LANE_W: f32 = 56.0;

// Lane center offsets from driver's perspective: rightmost to leftmost
// (all on one side of the road centerline).
// Each center is at 0.5, 1.5, 2.5 lane widths from centerline.
pub const LANE_OFFSETS: [f32; 3] = [
    2.5 * LANE_W,
    1.5 * LANE_W,
    0.5 * LANE_W,
];

#[derive(Clone, Debug)]
pub struct Vehicle {
    pub approach:  Approach,
    pub route:     Route,
    pub position:  (f32, f32),
    pub velocity:  f32,
    pub elapsed:   f32,
    pub heading:   (f32, f32),
    pub turned:    bool,
    pub turning:   bool,
    pub turn_t:    f32,
    pub turn_from: (f32, f32),
    pub turn_to:   (f32, f32),
    pub stopped_once: bool,
    pub stop_hold: f32,
}

impl Vehicle {
    pub fn new(
        approach: Approach,
        route: Route,
        position: (f32, f32),
        velocity: f32,
    ) -> Self {
        Self {
            approach, route, position, velocity,
            elapsed: 0.0,
            heading: heading_for_approach(approach),
            turned: false,
            turning: false,
            turn_t: 0.0,
            turn_from: heading_for_approach(approach),
            turn_to: heading_for_approach(approach),
            stopped_once: false,
            stop_hold: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        const TURN_DURATION: f32 = 0.34;

        self.elapsed += dt;
        if !self.turned && !self.turning && self.route != Route::Straight {
            if self.should_turn() {
                let new_heading = turn_heading(self.approach, self.route);
                self.turning = true;
                self.turn_t = 0.0;
                self.turn_from = self.heading;
                self.turn_to = new_heading;
            }
        }

        if self.turning {
            self.turn_t += (dt / TURN_DURATION).clamp(0.0, 1.0);
            let t = smoothstep(self.turn_t.clamp(0.0, 1.0));
            self.heading = normalize(lerp2(self.turn_from, self.turn_to, t));
            if self.turn_t >= 1.0 {
                self.turning = false;
                self.turned = true;
                self.heading = self.turn_to;
            }
        }

        self.position.0 += self.heading.0 * self.velocity * dt;
        self.position.1 += self.heading.1 * self.velocity * dt;
    }

    pub fn is_within_bounds(&self, limit: f32) -> bool {
        self.position.0.abs() <= limit && self.position.1.abs() <= limit
    }

    fn should_turn(&self) -> bool {
        let trigger: f32 = match self.route {
            Route::Right    => 1.0 * LANE_W,
            Route::Left     => 1.0 * LANE_W,
            Route::Straight => 0.0,
        };
        match self.approach {
            Approach::North => self.position.1 >= -trigger,
            Approach::South => self.position.1 <= trigger,
            Approach::East  => self.position.0 <= trigger,
            Approach::West  => self.position.0 >= -trigger,
        }
    }
}

fn turn_heading(approach: Approach, route: Route) -> (f32, f32) {
    match route {
        Route::Straight => heading_for_approach(approach),
        Route::Right => match approach {
            Approach::North => (-1.0,  0.0), // down  → right turn → go west
            Approach::South => ( 1.0,  0.0), // up    → right turn → go east
            Approach::East  => ( 0.0, -1.0), // left  → right turn → go north
            Approach::West  => ( 0.0,  1.0), // right → right turn → go south
        },
        Route::Left => match approach {
            Approach::North => ( 1.0,  0.0), // down  → left turn → go east
            Approach::South => (-1.0,  0.0), // up    → left turn → go west
            Approach::East  => ( 0.0,  1.0), // left  → left turn → go south
            Approach::West  => ( 0.0, -1.0), // right → left turn → go north
        },
    }
}

fn lerp2(a: (f32, f32), b: (f32, f32), t: f32) -> (f32, f32) {
    (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t)
}

fn normalize(v: (f32, f32)) -> (f32, f32) {
    let len = (v.0 * v.0 + v.1 * v.1).sqrt();
    if len <= f32::EPSILON {
        (0.0, 0.0)
    } else {
        (v.0 / len, v.1 / len)
    }
}

fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

pub fn heading_for_approach(approach: Approach) -> (f32, f32) {
    match approach {
        Approach::North => (0.0,  1.0),
        Approach::South => (0.0, -1.0),
        Approach::East  => (-1.0, 0.0),
        Approach::West  => ( 1.0, 0.0),
    }
}