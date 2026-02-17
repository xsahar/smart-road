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
            .flat_map(|lane| lane.vehicles.iter())
            .any(|vehicle| is_in_intersection(vehicle.position, INTERSECTION_HALF));
        let mut intersection_reserved = false;
        for lane in &mut self.lanes {
            let count = lane.vehicles.len();
            let mut order: Vec<usize> = (0..count).collect();
            order.sort_by(|&a, &b| {
                let sa = forward_progress(&lane.vehicles[a]);
                let sb = forward_progress(&lane.vehicles[b]);
                sb.partial_cmp(&sa).unwrap_or(Ordering::Equal)
            });

            let mut ahead_progress = None;
            for index in order {
                let progress = forward_progress(&lane.vehicles[index]);
                let mut should_move = true;
                if let Some(prev_progress) = ahead_progress {
                    if prev_progress - progress < SAFE_GAP {
                        should_move = false;
                        self.stats.record_close_call();
                    }
                }

                if should_move
                    && !is_in_intersection(lane.vehicles[index].position, INTERSECTION_HALF)
                    && would_enter_intersection(&lane.vehicles[index], dt, INTERSECTION_HALF)
                {
                    if intersection_occupied || intersection_reserved {
                        should_move = false;
                        self.stats.record_close_call();
                    } else {
                        intersection_reserved = true;
                    }
                }

                if should_move {
                    lane.vehicles[index].update(dt);
                }

                ahead_progress = Some(forward_progress(&lane.vehicles[index]));
            }

            let mut remaining = Vec::with_capacity(lane.vehicles.len());
            for vehicle in lane.vehicles.drain(..) {
                if vehicle.is_within_bounds(DESPAWN_LIMIT) {
                    remaining.push(vehicle);
                } else {
                    self.stats.record_exit(vehicle.elapsed);
                }
            }
            lane.vehicles = remaining;
        }
    }

    fn spawn_vehicle(&mut self, approach: Approach) {
        let route = self.next_random_route();
        let position = match approach {
            Approach::North => (0.0, -450.0),
            Approach::South => (0.0, 450.0),
            Approach::East => (450.0, 0.0),
            Approach::West => (-450.0, 0.0),
        };
        let vehicle = Vehicle::new(approach, route, position, 120.0);
        self.stats.record_spawn(vehicle.velocity);
        if let Some(lane) = self.lanes.iter_mut().find(|lane| lane.approach == approach) {
            lane.vehicles.push(vehicle);
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
