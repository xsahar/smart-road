// Simulation state: lanes, vehicles, and per-frame updates.
pub mod lane;
pub mod vehicle;

use lane::Lane;
use vehicle::{Approach, Route, Vehicle};
use std::cmp::Ordering;
use crate::stats::Stats;

pub struct Simulation {
    pub lanes: Vec<Lane>,
    pub stats: Stats,
    pending_spawns: Vec<Approach>,
    rng_state: u32,
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
            stats: Stats::new(),
            pending_spawns: Vec::new(),
            rng_state: 0x1234ABCD,
        }
    }

    pub fn request_spawn(&mut self, approach: Approach) {
        self.pending_spawns.push(approach);
    }

    pub fn request_spawn_random(&mut self) {
        let approach = self.next_random_approach();
        self.pending_spawns.push(approach);
    }

    pub fn update(&mut self, dt: f32) {
        let mut spawns = Vec::new();
        std::mem::swap(&mut spawns, &mut self.pending_spawns);
        for approach in spawns {
            self.spawn_vehicle(approach);
        }

        const DESPAWN_LIMIT: f32 = 520.0;
        const SAFE_GAP: f32 = 36.0;
        const INTERSECTION_HALF: f32 = 110.0;
        let intersection_occupied = self
            .lanes
            .iter()
            .flat_map(|lane| lane.incoming.iter().flat_map(|v| v.iter()))
            .any(|vehicle| is_in_intersection(vehicle.position, INTERSECTION_HALF));
        let mut intersection_reserved = false;
        // Build a temporary list of planned moves so we can reason about
        // same-lane spacing and cross-lane collisions without mutable borrow issues.
        #[derive(Clone)]
        struct Planned {
            lane_idx: usize,
            sublane_idx: usize,
            vehicle_idx: usize,
            approach: Approach,
            position: (f32, f32),
            heading: (f32, f32),
            velocity: f32,
            next: (f32, f32),
            allow_enter: bool,
            progress: f32,
        }

        let mut planned: Vec<Planned> = Vec::new();
        for (lane_idx, lane) in self.lanes.iter().enumerate() {
            for sublane_idx in 0..3usize {
                for (vehicle_idx, vehicle) in lane.incoming[sublane_idx].iter().enumerate() {
                    let progress = forward_progress(vehicle);
                    let next = (
                        vehicle.position.0 + vehicle.heading.0 * vehicle.velocity * dt,
                        vehicle.position.1 + vehicle.heading.1 * vehicle.velocity * dt,
                    );
                    planned.push(Planned {
                        lane_idx,
                        sublane_idx,
                        vehicle_idx,
                        approach: vehicle.approach,
                        position: vehicle.position,
                        heading: vehicle.heading,
                        velocity: vehicle.velocity,
                        next,
                        allow_enter: false,
                        progress,
                    });
                }
            }
        }

        // Arbitration: if multiple vehicles would enter the intersection next frame,
        // allow only one to proceed (choose the one with highest forward progress,
        // tie-break by approach priority). This prevents deadlock where all yield.
        if !intersection_occupied {
            // collect indices that would enter
            let enter_idxs: Vec<usize> = planned
                .iter()
                .enumerate()
                .filter(|(_, p)| {
                    let was_in = is_in_intersection(p.position, INTERSECTION_HALF);
                    let will_in = is_in_intersection(p.next, INTERSECTION_HALF);
                    !was_in && will_in
                })
                .map(|(i, _)| i)
                .collect();

            if enter_idxs.len() > 1 {
                // approach priority mapping (higher wins on tie): North, East, South, West
                let priority = |a: Approach| match a {
                    Approach::North => 3,
                    Approach::East => 2,
                    Approach::South => 1,
                    Approach::West => 0,
                };

                // pick winner index
                let mut winner = enter_idxs[0];
                for &i in &enter_idxs[1..] {
                    let pw = &planned[winner];
                    let pi = &planned[i];
                    if pi.progress > pw.progress || (pi.progress == pw.progress && priority(pi.approach) > priority(pw.approach)) {
                        winner = i;
                    }
                }
                // others yield
                for &i in &enter_idxs {
                    if i == winner { continue; }
                    if planned[i].velocity > 0.0 {
                        planned[i].velocity = 0.0;
                        planned[i].next = planned[i].position;
                        self.stats.record_close_call();
                    }
                }
                // mark winner as allowed to enter, and reserve the intersection so others won't
                planned[winner].allow_enter = true;
                intersection_reserved = true;
            }
        }

        // Same-lane safe distance: use a braking model so followers reduce speed smoothly
        // instead of stopping early. Compute allowed speed so vehicles don't close within
        // MIN_GAP after this timestep, and limit deceleration by MAX_BRAKE.
        const MIN_GAP: f32 = SAFE_GAP; // reuse existing constant
        // MAX_BRAKE is defined below; duplicate here for clarity
        const MAX_BRAKE_LOCAL: f32 = 300.0;
        for lane_idx in 0..self.lanes.len() {
            for sublane_idx in 0..3usize {
                // collect indices in this lane+sublane and sort by progress descending (front to back)
                let mut idxs: Vec<usize> = planned
                    .iter()
                    .enumerate()
                    .filter(|(_, p)| p.lane_idx == lane_idx && p.sublane_idx == sublane_idx)
                    .map(|(i, _)| i)
                    .collect();
            idxs.sort_by(|&a, &b| planned[b].progress.partial_cmp(&planned[a].progress).unwrap_or(Ordering::Equal));
                for w in 0..idxs.len() {
                    if w == 0 { continue; }
                    let cur_i = idxs[w];
                    let ahead_i = idxs[w - 1];

                // Copy ahead and current scalar data to avoid simultaneous borrows
                let ahead_pos = planned[ahead_i].position;
                let ahead_vel = planned[ahead_i].velocity;
                let ahead_approach = planned[ahead_i].approach;
                let cur_pos = planned[cur_i].position;
                let cur_approach = planned[cur_i].approach;
                let cur_vel = planned[cur_i].velocity;

                // scalar forward coordinates for current and ahead (using positions)
                let ahead_progress = match ahead_approach {
                    Approach::North => ahead_pos.1,
                    Approach::South => -ahead_pos.1,
                    Approach::East => -ahead_pos.0,
                    Approach::West => ahead_pos.0,
                };
                let cur_progress = match cur_approach {
                    Approach::North => cur_pos.1,
                    Approach::South => -cur_pos.1,
                    Approach::East => -cur_pos.0,
                    Approach::West => cur_pos.0,
                };

                let gap = ahead_progress - cur_progress - MIN_GAP;
                // predicted gap after dt if speeds unchanged
                let predicted_gap = gap + (ahead_vel - cur_vel) * dt;

                if predicted_gap < 0.0 {
                    // need to reduce cur.velocity so predicted_gap >= 0
                    // allowed_speed satisfies: gap + (ahead.velocity - allowed_speed)*dt >= 0
                    let allowed_speed = (ahead_vel + gap / dt).max(0.0);
                    // limit deceleration per frame
                    let decel = MAX_BRAKE_LOCAL * dt;
                    // now mutably borrow current planned entry and apply braking
                    let cur_mut = &mut planned[cur_i];
                    let new_vel = if allowed_speed < cur_mut.velocity {
                        (cur_mut.velocity - decel).max(allowed_speed)
                    } else {
                        cur_mut.velocity
                    };
                    if new_vel < cur_mut.velocity {
                        cur_mut.velocity = new_vel;
                        cur_mut.next = (
                            cur_mut.position.0 + cur_mut.heading.0 * cur_mut.velocity * dt,
                            cur_mut.position.1 + cur_mut.heading.1 * cur_mut.velocity * dt,
                        );
                        self.stats.record_close_call();
                    }
                }
                }
            }
        }

    // Inter-lane collision avoidance: iterative resolution. If two vehicles' next
    // positions would be closer than COLLIDE_DIST, the one with lower progress yields.
    // Use a smaller collision distance to avoid flagging adjacent lane vehicles and
    // apply a max braking rate for smooth deceleration instead of hard stops.
    const COLLIDE_DIST: f32 = 28.0;
    const MAX_BRAKE: f32 = 300.0; // px/s^2, how fast a vehicle can reduce speed per second
    const CRUISE_SPEED: f32 = 120.0;
    const MAX_ACCEL: f32 = 200.0; // px/s^2
    const MIN_CRAWL: f32 = 6.0; // small crawl speed when blocked
        // iterate a few times to settle dependencies
        for _iter in 0..4 {
            let mut changed = false;
            for i in 0..planned.len() {
                for j in (i + 1)..planned.len() {
                    let pi = &planned[i];
                    let pj = &planned[j];
                    let dx = pi.next.0 - pj.next.0;
                    let dy = pi.next.1 - pj.next.1;
                    if (dx * dx + dy * dy) < (COLLIDE_DIST * COLLIDE_DIST) {
                        // choose which should yield: lower forward progress (less advanced) yields
                        if pi.progress < pj.progress {
                            // reduce i
                            let cur_vel = planned[i].velocity;
                            let decel = MAX_BRAKE * dt;
                            let new_vel = (cur_vel - decel).max(0.0);
                            if new_vel < cur_vel {
                                planned[i].velocity = new_vel;
                                planned[i].next = (
                                    planned[i].position.0 + planned[i].heading.0 * planned[i].velocity * dt,
                                    planned[i].position.1 + planned[i].heading.1 * planned[i].velocity * dt,
                                );
                                changed = true;
                                self.stats.record_close_call();
                            }
                        } else {
                            let cur_vel = planned[j].velocity;
                            let decel = MAX_BRAKE * dt;
                            let new_vel = (cur_vel - decel).max(0.0);
                            if new_vel < cur_vel {
                                planned[j].velocity = new_vel;
                                planned[j].next = (
                                    planned[j].position.0 + planned[j].heading.0 * planned[j].velocity * dt,
                                    planned[j].position.1 + planned[j].heading.1 * planned[j].velocity * dt,
                                );
                                changed = true;
                                self.stats.record_close_call();
                            }
                        }
                    }
                }
            }
            if !changed { break; }
        }

        // Apply planned velocities and update vehicles. Also handle despawn.
        // After braking/collision resolution, allow vehicles to accelerate back toward cruise
        // when space permits, to avoid permanently stuck low speeds.
        for idx in 0..planned.len() {
            // find the vehicle ahead in the same lane (if any)
            let mut ahead_idx: Option<usize> = None;
            for j in 0..planned.len() {
                if planned[j].lane_idx == planned[idx].lane_idx && planned[j].progress > planned[idx].progress {
                    if ahead_idx.is_none() || planned[j].progress < planned[ahead_idx.unwrap()].progress {
                        ahead_idx = Some(j);
                    }
                }
            }

            // compute available gap (using positions) to decide if we can accel
            let available = if let Some(ai) = ahead_idx {
                let ahead_prog = match planned[ai].approach {
                    Approach::North => planned[ai].position.1,
                    Approach::South => -planned[ai].position.1,
                    Approach::East => -planned[ai].position.0,
                    Approach::West => planned[ai].position.0,
                };
                let cur_prog = match planned[idx].approach {
                    Approach::North => planned[idx].position.1,
                    Approach::South => -planned[idx].position.1,
                    Approach::East => -planned[idx].position.0,
                    Approach::West => planned[idx].position.0,
                };
                ahead_prog - cur_prog - MIN_GAP
            } else {
                // no one ahead: very large available space
                1e6
            };

            if planned[idx].velocity < CRUISE_SPEED && available > 2.0 {
                let accel = MAX_ACCEL * dt;
                planned[idx].velocity = (planned[idx].velocity + accel).min(CRUISE_SPEED);
                planned[idx].next = (
                    planned[idx].position.0 + planned[idx].heading.0 * planned[idx].velocity * dt,
                    planned[idx].position.1 + planned[idx].heading.1 * planned[idx].velocity * dt,
                );
            }
        }

        for p in planned.into_iter() {
            if let Some(lane) = self.lanes.get_mut(p.lane_idx) {
                if p.sublane_idx < lane.incoming.len() {
                    let vec = &mut lane.incoming[p.sublane_idx];
                    if let Some(vehicle) = vec.get_mut(p.vehicle_idx) {
                        vehicle.velocity = p.velocity;
                    // If this vehicle is going to enter the intersection, ensure only one is allowed
                    // to enter per frame. Use the planned next position (`p.next`) and the
                    // arbitration `allow_enter` flag so the selected winner is permitted.
                    let will_enter = !is_in_intersection(vehicle.position, INTERSECTION_HALF)
                        && is_in_intersection(p.next, INTERSECTION_HALF);
                    if will_enter {
                        if intersection_occupied {
                            // someone already inside - stop
                            vehicle.velocity = 0.0;
                            self.stats.record_close_call();
                        } else if intersection_reserved {
                            // reserved this frame by arbitration; allow only the winner
                            if !p.allow_enter {
                                // allow a small crawl so vehicles can approach the intersection slowly
                                vehicle.velocity = vehicle.velocity.min(MIN_CRAWL);
                                self.stats.record_close_call();
                            } else {
                                // winner proceeds; reservation already set earlier
                            }
                        } else {
                            // no reservation and not occupied: allow and reserve
                            intersection_reserved = true;
                        }
                    }

                        // Finally update position according to the possibly reduced velocity.
                        vehicle.update(dt);
                    }
                }
            }
        }

        // Despawn and collect remaining vehicles per lane (per sublane)
        for lane in &mut self.lanes {
            for sublane in 0..lane.incoming.len() {
                let mut remaining = Vec::with_capacity(lane.incoming[sublane].len());
                for vehicle in lane.incoming[sublane].drain(..) {
                    if vehicle.is_within_bounds(DESPAWN_LIMIT) {
                        remaining.push(vehicle);
                    } else {
                        self.stats.record_exit(vehicle.elapsed);
                    }
                }
                lane.incoming[sublane] = remaining;
            }
        }
    }

    fn spawn_vehicle(&mut self, approach: Approach) {
        let route = self.next_random_route();
        // Determine lane index for the route: 0 = Right, 1 = Straight, 2 = Left
        let lane_index: u8 = match route {
            Route::Right => 0,
            Route::Straight => 1,
            Route::Left => 2,
        };

        // Base spawn positions per approach (center of incoming road)
        let base = match approach {
            Approach::North => (0.0, -450.0),
            Approach::South => (0.0, 450.0),
            Approach::East => (450.0, 0.0),
            Approach::West => (-450.0, 0.0),
        };

        // Lateral offset perpendicular to heading so vehicles occupy distinct lane paths
        const LANE_SPACING: f32 = 28.0;
        let heading = match approach {
            Approach::North => (0.0, 1.0),
            Approach::South => (0.0, -1.0),
            Approach::East => (-1.0, 0.0),
            Approach::West => (1.0, 0.0),
        };
        // left vector (perp to heading)
        let left = (-heading.1, heading.0);
        let lateral = match lane_index {
            0 => (-left.0 * LANE_SPACING, -left.1 * LANE_SPACING), // right lane
            1 => (0.0, 0.0),                                       // straight
            2 => (left.0 * LANE_SPACING, left.1 * LANE_SPACING),   // left lane
            _ => (0.0, 0.0),
        };

        let spawn_pos = (base.0 + lateral.0, base.1 + lateral.1);

        // Enforce spawn spacing: find nearest vehicle in same approach & sublane and skip spawn if too close
        const SPAWN_GAP: f32 = 100.0;
        if let Some(lane) = self.lanes.iter().find(|lane| lane.approach == approach) {
            let vec = &lane.incoming[lane_index as usize];
            let spawn_prog = forward_progress(&Vehicle::new(approach, route, lane_index, spawn_pos, 0.0));
            let too_close = vec.iter().any(|v| (forward_progress(v) - spawn_prog).abs() < SPAWN_GAP);
            if too_close {
                return; // skip spawn this frame to preserve spacing
            }
        }

        let vehicle = Vehicle::new(approach, route, lane_index, spawn_pos, 120.0);
        self.stats.record_spawn(vehicle.velocity);
        if let Some(lane) = self.lanes.iter_mut().find(|lane| lane.approach == approach) {
            lane.incoming[lane_index as usize].push(vehicle);
        }
    }

    pub fn stats(&self) -> &Stats {
        &self.stats
    }

    fn next_random_approach(&mut self) -> Approach {
        match (self.next_random_u32() % 4) as u8 {
            0 => Approach::North,
            1 => Approach::South,
            2 => Approach::East,
            _ => Approach::West,
        }
    }

    fn next_random_route(&mut self) -> Route {
        match (self.next_random_u32() % 3) as u8 {
            0 => Route::Right,
            1 => Route::Straight,
            _ => Route::Left,
        }
    }

    fn next_random_u32(&mut self) -> u32 {
        self.rng_state = self
            .rng_state
            .wrapping_mul(1664525)
            .wrapping_add(1013904223);
        self.rng_state
    }
}

fn forward_progress(vehicle: &Vehicle) -> f32 {
    match vehicle.approach {
        Approach::North => vehicle.position.1,
        Approach::South => -vehicle.position.1,
        Approach::East => -vehicle.position.0,
        Approach::West => vehicle.position.0,
    }
}

fn is_in_intersection(position: (f32, f32), half_size: f32) -> bool {
    position.0.abs() <= half_size && position.1.abs() <= half_size
}

fn would_enter_intersection(vehicle: &Vehicle, dt: f32, half_size: f32) -> bool {
    let next = (
        vehicle.position.0 + vehicle.heading.0 * vehicle.velocity * dt,
        vehicle.position.1 + vehicle.heading.1 * vehicle.velocity * dt,
    );
    !is_in_intersection(vehicle.position, half_size) && is_in_intersection(next, half_size)
}
