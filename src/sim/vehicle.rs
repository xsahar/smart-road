// Vehicle types and per-frame physics including curved intersection turns.
// All position changes use:  x += velocity * delta_time
// There are no raw `+= 1` or lerp-fraction position jumps.

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
    pub velocity:     f32,          // forward speed  (px/s)
    pub elapsed:      f32,
    pub heading:      (f32, f32),   // unit vector, forward direction
    pub turned:       bool,
    pub turning:      bool,
    pub turn_t:       f32,          // 0..1 turn progress
    pub turn_from:    (f32, f32),
    pub turn_to:      (f32, f32),
    pub stopped_once: bool,
    pub stop_hold:    f32,
    /// Lateral speed used to slide the vehicle into the outgoing lane
    /// during a turn.  Computed once when the turn starts; applied as
    ///   lateral_offset += lateral_velocity * dt
    /// so the movement is always velocity × delta_time.
    lateral_velocity: f32,
    /// Target lateral coordinate once the turn completes.
    lateral_target:   f32,
    /// Which axis is lateral for this turn (true = Y axis, false = X axis).
    lateral_axis_y:   bool,
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
            elapsed:          0.0,
            heading:          heading_for_approach(approach),
            turned:           false,
            turning:          false,
            turn_t:           0.0,
            turn_from:        heading_for_approach(approach),
            turn_to:          heading_for_approach(approach),
            stopped_once:     false,
            stop_hold:        0.0,
            lateral_velocity: 0.0,
            lateral_target:   0.0,
            lateral_axis_y:   false,
        }
    }

    pub fn update(&mut self, dt: f32) {
        const TURN_DURATION: f32 = 0.34;  // seconds to complete the heading rotation

        self.elapsed += dt;

        // ── Detect when to begin turning ─────────────────────────────────────
        if !self.turned && !self.turning && self.route != Route::Straight {
            if self.should_turn() {
                let new_heading = turn_heading(self.approach, self.route);
                self.turning  = true;
                self.turn_t   = 0.0;
                self.turn_from = self.heading;
                self.turn_to   = new_heading;

                // Compute the lateral lane target and a constant lateral speed
                // so the vehicle slides across exactly one lane-change distance
                // over TURN_DURATION seconds:
                //   lateral_velocity = distance_to_target / TURN_DURATION
                let (axis_y, target) = outgoing_lane_target(new_heading, self.route);
                self.lateral_axis_y = axis_y;
                self.lateral_target = target;

                let current_lateral = if axis_y { self.position.1 } else { self.position.0 };
                let distance        = target - current_lateral;
                // lateral_velocity carries the sign (positive or negative).
                self.lateral_velocity = distance / TURN_DURATION;
            }
        }

        //  Heading rotation + lateral slide during turn 
        if self.turning {
            self.turn_t += dt / TURN_DURATION;

            // Interpolate heading angle (rotational part).
            let t = smoothstep(self.turn_t.clamp(0.0, 1.0));
            self.heading = normalize(lerp2(self.turn_from, self.turn_to, t));

            // Lateral slide: x += lateral_velocity * dt  (pure physics formula)
            if self.lateral_axis_y {
                self.position.1 += self.lateral_velocity * dt;
                // Clamp so we never overshoot the target lane.
                if self.lateral_velocity > 0.0 {
                    self.position.1 = self.position.1.min(self.lateral_target);
                } else {
                    self.position.1 = self.position.1.max(self.lateral_target);
                }
            } else {
                self.position.0 += self.lateral_velocity * dt;
                if self.lateral_velocity > 0.0 {
                    self.position.0 = self.position.0.min(self.lateral_target);
                } else {
                    self.position.0 = self.position.0.max(self.lateral_target);
                }
            }

            // Finish turn: snap heading and pin lateral position exactly.
            if self.turn_t >= 1.0 {
                self.turning = false;
                self.turned  = true;
                self.heading = self.turn_to;
                self.lateral_velocity = 0.0;
                // Final snap: place on the exact target (removes any float drift).
                if self.lateral_axis_y {
                    self.position.1 = self.lateral_target;
                } else {
                    self.position.0 = self.lateral_target;
                }
            }
        }

        //  Forward movement: x += velocity * delta_time 
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
            Approach::South => self.position.1 <=  trigger,
            Approach::East  => self.position.0 <=  trigger,
            Approach::West  => self.position.0 >= -trigger,
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

/// Returns (axis_is_y, target_coordinate) for the outgoing lane centre.
fn outgoing_lane_target(heading: (f32, f32), route: Route) -> (bool, f32) {
    let lane_offset = match route {
        Route::Right    => 2.5 * LANE_W,
        Route::Straight => 1.5 * LANE_W,
        Route::Left     => 0.5 * LANE_W,
    };
    // After the turn, heading tells us the new direction of travel.
    // The lateral axis is perpendicular to the new heading.
    if heading.0.abs() > heading.1.abs() {
        // Moving horizontally → lateral axis is Y.
        let y = if heading.0 > 0.0 { lane_offset } else { -lane_offset };
        (true, y)
    } else {
        // Moving vertically → lateral axis is X.
        let x = if heading.1 > 0.0 { -lane_offset } else { lane_offset };
        (false, x)
    }
}

fn lerp2(a: (f32, f32), b: (f32, f32), t: f32) -> (f32, f32) {
    (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t)
}

fn normalize(v: (f32, f32)) -> (f32, f32) {
    let len = (v.0 * v.0 + v.1 * v.1).sqrt();
    if len <= f32::EPSILON { (0.0, 0.0) } else { (v.0 / len, v.1 / len) }
}

fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

pub fn heading_for_approach(approach: Approach) -> (f32, f32) {
    match approach {
        Approach::North => ( 0.0,  1.0),
        Approach::South => ( 0.0, -1.0),
        Approach::East  => (-1.0,  0.0),
        Approach::West  => ( 1.0,  0.0),
    }
}