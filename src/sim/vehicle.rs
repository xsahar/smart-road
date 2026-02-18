// Vehicle types and per-frame physics including curved intersection turns.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Route { Right, Straight, Left }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Approach { North, South, East, West }


pub const LANE_W: f32 = 30.0;

pub const LANE_OFFSETS: [f32; 3] = [2.5 * LANE_W, 1.5 * LANE_W, 0.5 * LANE_W];

#[derive(Clone, Debug)]
pub struct Vehicle {
    pub approach:  Approach,
    pub route:     Route,
    pub position:  (f32, f32),
    pub velocity:  f32,
    pub elapsed:   f32,
    pub heading:   (f32, f32),
    pub lane:      u8,           // 0=Right, 1=Straight, 2=Left
    pub turned:    bool,
}

impl Vehicle {
    pub fn new(
        approach: Approach,
        route: Route,
        lane: u8,
        position: (f32, f32),
        velocity: f32,
    ) -> Self {
        Self {
            approach, route, position, velocity,
            elapsed: 0.0,
            heading: heading_for_approach(approach),
            lane, turned: false,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.elapsed += dt;
        if !self.turned && self.route != Route::Straight {
            if self.should_turn() {
                let (new_heading, new_lateral) = turn_result(self.approach, self.route);
                self.heading = new_heading;
                self.turned  = true;

                snap_to_outgoing_lane(self, new_lateral);
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
            Route::Right    =>  2.0 * LANE_W,
            Route::Left     => -1.0 * LANE_W,
            Route::Straight =>  0.0,
        };
        match self.approach {
            Approach::North => self.position.1 >=  -trigger,
            Approach::South => self.position.1 <=   trigger,
            Approach::East  => self.position.0 <=   trigger,
            Approach::West  => self.position.0 >= - trigger,
        }
    }
}


fn snap_to_outgoing_lane(v: &mut Vehicle, outgoing_lateral: f32) {
    let lx = -v.heading.1;
    let ly =  v.heading.0;
    let fwd_dist = v.position.0 * v.heading.0 + v.position.1 * v.heading.1;
    v.position.0 = fwd_dist * v.heading.0 + lx * outgoing_lateral;
    v.position.1 = fwd_dist * v.heading.1 + ly * outgoing_lateral;
}

fn turn_result(approach: Approach, route: Route) -> ((f32, f32), f32) {

    let exit_offset = 2.5 * LANE_W;

    let new_heading = match route {
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
    };
    (new_heading, exit_offset)
}

pub fn heading_for_approach(approach: Approach) -> (f32, f32) {
    match approach {
        Approach::North => (0.0,  1.0),
        Approach::South => (0.0, -1.0),
        Approach::East  => (-1.0, 0.0),
        Approach::West  => ( 1.0, 0.0),
    }
}

pub fn turn_heading(approach: Approach, route: Route) -> (f32, f32) {
    turn_result(approach, route).0
}