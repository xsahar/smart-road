// Simulation state: lanes, vehicles, and per-frame updates.
pub mod lane;
pub mod vehicle;

use lane::Lane;
use vehicle::{Approach, Route, Vehicle};
use std::cmp::Ordering;
use crate::stats::Stats;

pub use vehicle::{LANE_W, LANE_OFFSETS};

pub struct Simulation {
    pub lanes:     Vec<Lane>,
    pub stats:     Stats,
    pending_spawns: Vec<Approach>,
    rng_state:     u32,
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

        const DESPAWN_LIMIT:       f32 = 540.0;
        const SAFE_GAP:            f32 = 36.0;
        const INTERSECTION_HALF:   f32 = 4.0 * LANE_W; // covers staggered turn triggers
        const MIN_GAP:             f32 = SAFE_GAP;
        const MAX_BRAKE:           f32 = 300.0;
        const CRUISE_SPEED:        f32 = 120.0;
        const MAX_ACCEL:           f32 = 200.0;
        const MIN_CRAWL:           f32 = 6.0;
        const COLLIDE_DIST:        f32 = 26.0;

        let intersection_occupied = self
            .lanes
            .iter()
            .flat_map(|lane| lane.incoming.iter().flat_map(|v| v.iter()))
            .any(|v| is_in_intersection(v.position, INTERSECTION_HALF));

        let mut intersection_reserved = false;

        #[derive(Clone)]
        struct Planned {
            lane_idx:    usize,
            sublane_idx: usize,
            vehicle_idx: usize,
            approach:    Approach,
            position:    (f32, f32),
            heading:     (f32, f32),
            velocity:    f32,
            next:        (f32, f32),
            allow_enter: bool,
            progress:    f32,
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
                        lane_idx, sublane_idx, vehicle_idx,
                        approach: vehicle.approach,
                        position: vehicle.position,
                        heading:  vehicle.heading,
                        velocity: vehicle.velocity,
                        next,
                        allow_enter: false,
                        progress,
                    });
                }
            }
        }

        if !intersection_occupied {
            let enter_idxs: Vec<usize> = planned
                .iter()
                .enumerate()
                .filter(|(_, p)| {
                    !is_in_intersection(p.position, INTERSECTION_HALF)
                        && is_in_intersection(p.next, INTERSECTION_HALF)
                })
                .map(|(i, _)| i)
                .collect();

            if enter_idxs.len() > 1 {
                let priority = |a: Approach| match a {
                    Approach::North => 3,
                    Approach::East  => 2,
                    Approach::South => 1,
                    Approach::West  => 0,
                };
                let mut winner = enter_idxs[0];
                for &i in &enter_idxs[1..] {
                    let pw = &planned[winner];
                    let pi = &planned[i];
                    if pi.progress > pw.progress
                        || (pi.progress == pw.progress && priority(pi.approach) > priority(pw.approach))
                    {
                        winner = i;
                    }
                }
                for &i in &enter_idxs {
                    if i == winner { continue; }
                    if planned[i].velocity > 0.0 {
                        planned[i].velocity = 0.0;
                        planned[i].next = planned[i].position;
                        self.stats.record_close_call();
                    }
                }
                planned[winner].allow_enter = true;
                intersection_reserved = true;
            }
        }

        for lane_idx in 0..self.lanes.len() {
            for sublane_idx in 0..3usize {
                let mut idxs: Vec<usize> = planned
                    .iter()
                    .enumerate()
                    .filter(|(_, p)| p.lane_idx == lane_idx && p.sublane_idx == sublane_idx)
                    .map(|(i, _)| i)
                    .collect();
                idxs.sort_by(|&a, &b| {
                    planned[b].progress.partial_cmp(&planned[a].progress)
                        .unwrap_or(Ordering::Equal)
                });
                for w in 1..idxs.len() {
                    let cur_i    = idxs[w];
                    let ahead_i  = idxs[w - 1];

                    let ahead_vel = planned[ahead_i].velocity;
                    let ahead_progress = planned[ahead_i].progress;
                    let cur_progress   = planned[cur_i].progress;
                    let cur_vel        = planned[cur_i].velocity;

                    let gap = ahead_progress - cur_progress - MIN_GAP;
                    let predicted_gap = gap + (ahead_vel - cur_vel) * dt;

                    if predicted_gap < 0.0 {
                        let allowed_speed = (ahead_vel + gap / dt).max(0.0);
                        let decel = MAX_BRAKE * dt;
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

        for _iter in 0..4 {
            let mut changed = false;
            for i in 0..planned.len() {
                for j in (i + 1)..planned.len() {
                    let dx = planned[i].next.0 - planned[j].next.0;
                    let dy = planned[i].next.1 - planned[j].next.1;
                    if dx * dx + dy * dy < COLLIDE_DIST * COLLIDE_DIST {
                        let yield_i = planned[i].progress < planned[j].progress;
                        let yi = if yield_i { i } else { j };
                        let cur_vel = planned[yi].velocity;
                        let decel   = MAX_BRAKE * dt;
                        let new_vel = (cur_vel - decel).max(0.0);
                        if new_vel < cur_vel {
                            planned[yi].velocity = new_vel;
                            planned[yi].next = (
                                planned[yi].position.0 + planned[yi].heading.0 * new_vel * dt,
                                planned[yi].position.1 + planned[yi].heading.1 * new_vel * dt,
                            );
                            changed = true;
                            self.stats.record_close_call();
                        }
                    }
                }
            }
            if !changed { break; }
        }

        for idx in 0..planned.len() {
            let available = {
                let mut best: Option<f32> = None;
                for j in 0..planned.len() {
                    if planned[j].lane_idx == planned[idx].lane_idx
                        && planned[j].progress > planned[idx].progress
                    {
                        let gap = planned[j].progress - planned[idx].progress - MIN_GAP;
                        if best.map_or(true, |b| gap < b) {
                            best = Some(gap);
                        }
                    }
                }
                best.unwrap_or(1e6)
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

                        let will_enter = !is_in_intersection(vehicle.position, INTERSECTION_HALF)
                            && is_in_intersection(p.next, INTERSECTION_HALF);

                        if will_enter {
                            if intersection_occupied {
                                vehicle.velocity = 0.0;
                                self.stats.record_close_call();
                            } else if intersection_reserved {
                                if !p.allow_enter {
                                    vehicle.velocity = vehicle.velocity.min(MIN_CRAWL);
                                    self.stats.record_close_call();
                                }
                            } else {
                                intersection_reserved = true;
                            }
                        }

                        vehicle.update(dt);
                    }
                }
            }
        }

        // ── Despawn + per-frame velocity tracking ────────────────────────────
        for lane in &mut self.lanes {
            for sublane in 0..lane.incoming.len() {
                let mut remaining = Vec::with_capacity(lane.incoming[sublane].len());
                for vehicle in lane.incoming[sublane].drain(..) {
                    self.stats.record_velocity(vehicle.velocity);
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
        let sublane: usize = match route {
            Route::Right    => 0,
            Route::Straight => 1,
            Route::Left     => 2,
        };

       
        let heading = vehicle::heading_for_approach(approach);
 
        let left = (-heading.1, heading.0);
        let lateral_offset = LANE_OFFSETS[sublane];
        let lateral = (left.0 * lateral_offset, left.1 * lateral_offset);

        const SPAWN_DIST: f32 = 450.0;
        let base = (-heading.0 * SPAWN_DIST + lateral.0, -heading.1 * SPAWN_DIST + lateral.1);

        // Enforce spawn spacing
        const SPAWN_GAP: f32 = 100.0;
        let dummy = Vehicle::new(approach, route, sublane as u8, base, 0.0);
        let spawn_prog = forward_progress(&dummy);
        if let Some(lane) = self.lanes.iter().find(|l| l.approach == approach) {
            let too_close = lane.incoming[sublane]
                .iter()
                .any(|v| (forward_progress(v) - spawn_prog).abs() < SPAWN_GAP);
            if too_close { return; }
        }

        let vehicle = Vehicle::new(approach, route, sublane as u8, base, 120.0);
        if let Some(lane) = self.lanes.iter_mut().find(|l| l.approach == approach) {
            lane.incoming[sublane].push(vehicle);
        }
    }

    pub fn stats(&self) -> &Stats { &self.stats }

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
        self.rng_state = self.rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
        self.rng_state
    }
}

fn forward_progress(vehicle: &Vehicle) -> f32 {
    match vehicle.approach {
        Approach::North =>  vehicle.position.1,
        Approach::South => -vehicle.position.1,
        Approach::East  => -vehicle.position.0,
        Approach::West  =>  vehicle.position.0,
    }
}

fn is_in_intersection(position: (f32, f32), half_size: f32) -> bool {
    position.0.abs() <= half_size && position.1.abs() <= half_size
}