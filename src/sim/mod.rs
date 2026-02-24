// Simulation state: lanes, vehicles, and per-frame updates.
// Uses a two-layer reservation system for intersection management.

pub mod lane;
pub mod vehicle;

use lane::Lane;
use vehicle::{Approach, Route, Vehicle};
use std::collections::{HashMap, HashSet};
use crate::stats::Stats;

pub use vehicle::{LANE_W, LANE_OFFSETS};

const INTERSECTION_HALF: f32 = 4.0 * LANE_W;

/// How many grid cells per axis cover the intersection.
const GRID_N: i32 = 8;

/// Width of one spatial tile in pixels.
const TILE_SIZE: f32 = (2.0 * INTERSECTION_HALF) / GRID_N as f32;

/// Duration of one time-step in seconds.
const TIME_STEP: f32 = 0.05;

/// How many time-steps we look ahead when building a reservation.
const LOOKAHEAD: usize = 40;

const VEHICLE_RADIUS: f32 = 0.55 * LANE_W;


type VehicleId = u64;

type Tile = (i32, i32);

type Slot = (Tile, usize);


#[derive(Clone)]
pub struct Reservation {
    pub vehicle_id: VehicleId,
    pub entry_time: f32,
    pub exit_time:  f32,
}


struct ReservationManager {
    reservations: Vec<Reservation>,
    reserved: HashMap<Slot, VehicleId>,
    confirmed: HashSet<VehicleId>,
    clock: f32,
    next_id: VehicleId,
}

impl ReservationManager {
    fn new() -> Self {
        Self {
            reservations: Vec::new(),
            reserved:     HashMap::new(),
            confirmed:    HashSet::new(),
            clock:        0.0,
            next_id:      1,
        }
    }


    fn tick(&mut self, dt: f32) {
        self.clock += dt;
        let current_step = self.current_step();
        // Expire old tile-slots.
        self.reserved.retain(|&(_, t), _| t >= current_step);
        // Expire time-interval reservations that have fully exited.
        self.reservations.retain(|r| r.exit_time > self.clock);
    }

    fn current_step(&self) -> usize {
        (self.clock / TIME_STEP) as usize
    }

    fn alloc_id(&mut self) -> VehicleId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn can_reserve(&self, entry: f32, exit: f32) -> bool {
        for r in &self.reservations {
            let overlap = entry < r.exit_time && exit > r.entry_time;
            if overlap {
                return false;
            }
        }
        true
    }

    fn reserve_interval(&mut self, vehicle_id: VehicleId, entry: f32, exit: f32) {
        self.reservations.push(Reservation { vehicle_id, entry_time: entry, exit_time: exit });
        self.confirmed.insert(vehicle_id);
    }


    fn request(&mut self, id: VehicleId, slots: &[Slot]) -> bool {
        for slot in slots {
            if let Some(&owner) = self.reserved.get(slot) {
                if owner != id {
                    return false; // conflict – deny
                }
            }
        }
        for &slot in slots {
            self.reserved.insert(slot, id);
        }
        self.confirmed.insert(id);
        true
    }

    fn release(&mut self, id: VehicleId) {
        self.reserved.retain(|_, owner| *owner != id);
        self.reservations.retain(|r| r.vehicle_id != id);
        self.confirmed.remove(&id);
    }

    fn has_confirmed(&self, id: VehicleId) -> bool {
        self.confirmed.contains(&id)
    }
}



fn tiles_at(pos: (f32, f32)) -> Vec<Tile> {
    let mut result = Vec::new();
    let r = VEHICLE_RADIUS + TILE_SIZE * 0.5;
    let col_lo = ((pos.0 - r + INTERSECTION_HALF) / TILE_SIZE).floor() as i32;
    let col_hi = ((pos.0 + r + INTERSECTION_HALF) / TILE_SIZE).floor() as i32;
    let row_lo = ((pos.1 - r + INTERSECTION_HALF) / TILE_SIZE).floor() as i32;
    let row_hi = ((pos.1 + r + INTERSECTION_HALF) / TILE_SIZE).floor() as i32;
    for c in col_lo..=col_hi {
        for rr in row_lo..=row_hi {
            if c >= 0 && c < GRID_N && rr >= 0 && rr < GRID_N {
                let tx = (c as f32 + 0.5) * TILE_SIZE - INTERSECTION_HALF;
                let ty = (rr as f32 + 0.5) * TILE_SIZE - INTERSECTION_HALF;
                let dx = tx - pos.0;
                let dy = ty - pos.1;
                if dx * dx + dy * dy <= r * r {
                    result.push((c, rr));
                }
            }
        }
    }
    result
}

fn build_reservation_slots(
    start_pos:  (f32, f32),
    heading:    (f32, f32),
    speed:      f32,
    start_step: usize,
) -> Vec<Slot> {
    let mut slots = Vec::new();
    let mut pos = start_pos;
    for i in 0..LOOKAHEAD {
        pos = (
            pos.0 + heading.0 * speed * TIME_STEP,
            pos.1 + heading.1 * speed * TIME_STEP,
        );
        for tile in tiles_at(pos) {
            slots.push((tile, start_step + i));
        }
        if pos.0.abs() > INTERSECTION_HALF + VEHICLE_RADIUS
            && pos.1.abs() > INTERSECTION_HALF + VEHICLE_RADIUS
        {
            break;
        }
    }
    slots.sort_unstable();
    slots.dedup();
    slots
}


pub struct Simulation {
    pub lanes:      Vec<Lane>,
    pub stats:      Stats,
    pending_spawns: Vec<Approach>,
    rng_state:      u32,
    approach_bag:   Vec<Approach>,
    bag_index:      usize,
    reservation:    ReservationManager,
    vehicle_ids:    HashMap<(usize, usize, usize), VehicleId>,
}

impl Simulation {
    pub fn new() -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| (d.as_nanos() as u64) ^ (d.as_secs() as u64))
            .unwrap_or(0x1234ABCD);
        let mut sim = Self {
            lanes: vec![
                Lane::new(Approach::North),
                Lane::new(Approach::South),
                Lane::new(Approach::East),
                Lane::new(Approach::West),
            ],
            stats:          Stats::new(),
            pending_spawns: Vec::new(),
            rng_state:      seed as u32,
            approach_bag:   Vec::new(),
            bag_index:      0,
            reservation:    ReservationManager::new(),
            vehicle_ids:    HashMap::new(),
        };
        sim.refill_approach_bag();
        sim
    }

    pub fn request_spawn(&mut self, approach: Approach) {
        self.pending_spawns.push(approach);
    }

    pub fn request_spawn_random(&mut self) {
        let approach = self.next_random_approach();
        self.pending_spawns.push(approach);
    }

    pub fn update(&mut self, dt: f32) {
        self.reservation.tick(dt);

        let mut spawns = Vec::new();
        std::mem::swap(&mut spawns, &mut self.pending_spawns);
        for approach in spawns {
            self.spawn_vehicle(approach);
        }

        const DESPAWN_LIMIT:  f32 = 540.0;
        const SAFE_GAP:       f32 = 1.4 * LANE_W;
        const MIN_GAP:        f32 = SAFE_GAP;
        const MAX_BRAKE:      f32 = 300.0;
        const CRUISE_SPEED:   f32 = 120.0;
        const MAX_ACCEL:      f32 = 200.0;
        const COLLIDE_DIST:   f32 = 0.9 * LANE_W;
        const STOP_LINE_DIST: f32 = 0.65 * LANE_W;
        const STOP_HOLD_TIME: f32 = 0.35;

        #[derive(Clone)]
        struct Planned {
            lane_idx:    usize,
            sublane_idx: usize,
            vehicle_idx: usize,
            position:    (f32, f32),
            heading:     (f32, f32),
            velocity:    f32,
            next:        (f32, f32),
            progress:    f32,
            vehicle_id:  VehicleId,
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
                    let key = (lane_idx, sublane_idx, vehicle_idx);
                    let vehicle_id = *self.vehicle_ids.entry(key)
                        .or_insert_with(|| self.reservation.alloc_id());
                    planned.push(Planned {
                        lane_idx, sublane_idx, vehicle_idx,
                        position: vehicle.position,
                        heading:  vehicle.heading,
                        velocity: vehicle.velocity,
                        next,
                        progress,
                        vehicle_id,
                    });
                }
            }
        }

        let current_step = self.reservation.current_step();
        let current_time = self.reservation.clock;

        for p in planned.iter_mut() {
            let in_intersection = is_in_intersection(p.position, INTERSECTION_HALF);
            let about_to_enter  = !in_intersection
                && is_in_intersection(p.next, INTERSECTION_HALF);

            if about_to_enter && !self.reservation.has_confirmed(p.vehicle_id) {
        
                let (entry_time, exit_time) =
                    compute_reservation(p.position, p.heading, p.velocity, current_time);

                if !self.reservation.can_reserve(entry_time, exit_time) {
                    p.velocity = 0.0;
                    p.next     = p.position;
                 
                } else {
                    //  space-time
                    let slots = build_reservation_slots(
                        p.position,
                        p.heading,
                        CRUISE_SPEED,
                        current_step,
                    );
                    let granted = self.reservation.request(p.vehicle_id, &slots);
                    if granted {
                        // Persist the time-interval alongside the tile slots.
                        self.reservation.reserve_interval(
                            p.vehicle_id,
                            entry_time,
                            exit_time,
                        );
                    } else {
                        // Tile conflict
                        p.velocity = 0.0;
                        p.next     = p.position;
                        self.stats.record_close_call();
                    }
                }
            }
        }

        //  Same-lane car-following
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
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                for w in 1..idxs.len() {
                    let cur_i   = idxs[w];
                    let ahead_i = idxs[w - 1];

                    let ahead_vel      = planned[ahead_i].velocity;
                    let ahead_progress = planned[ahead_i].progress;
                    let cur_progress   = planned[cur_i].progress;
                    let cur_vel        = planned[cur_i].velocity;

                    let gap           = ahead_progress - cur_progress - MIN_GAP;
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

                        if !vehicle.stopped_once && !is_in_intersection(vehicle.position, INTERSECTION_HALF) {
                            let stop_dist =
                                distance_to_intersection_edge(vehicle.position, INTERSECTION_HALF);
                            if stop_dist <= STOP_LINE_DIST {
                                if vehicle.stop_hold < STOP_HOLD_TIME {
                                    vehicle.stop_hold += dt;
                                    vehicle.velocity = 0.0;
                                } else {
                                    vehicle.stopped_once = true;
                                }
                            }
                        }

                        vehicle.update(dt);
                    }
                }
            }
        }

        let mut ids_to_release: Vec<VehicleId> = Vec::new();

        for (lane_idx, lane) in self.lanes.iter_mut().enumerate() {
            for sublane_idx in 0..lane.incoming.len() {
                let mut remaining = Vec::with_capacity(lane.incoming[sublane_idx].len());
                let mut new_idx = 0usize;
                for (old_idx, vehicle) in lane.incoming[sublane_idx].drain(..).enumerate() {
                    self.stats.record_velocity(vehicle.velocity);
                    if vehicle.is_within_bounds(DESPAWN_LIMIT) {
                        let old_key = (lane_idx, sublane_idx, old_idx);
                        let new_key = (lane_idx, sublane_idx, new_idx);
                        if let Some(id) = self.vehicle_ids.remove(&old_key) {
                            self.vehicle_ids.insert(new_key, id);
                        }
                        remaining.push(vehicle);
                        new_idx += 1;
                    } else {
                        self.stats.record_exit(vehicle.elapsed);
                        let key = (lane_idx, sublane_idx, old_idx);
                        if let Some(id) = self.vehicle_ids.remove(&key) {
                            ids_to_release.push(id);
                        }
                    }
                }
                lane.incoming[sublane_idx] = remaining;
            }
        }

        for id in ids_to_release {
            self.reservation.release(id);
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
        let right   = (-heading.1, heading.0);
        let lateral_offset = LANE_OFFSETS[sublane];
        let lateral = (right.0 * lateral_offset, right.1 * lateral_offset);

        const SPAWN_DIST: f32 = 450.0;
        let base = (
            -heading.0 * SPAWN_DIST + lateral.0,
            -heading.1 * SPAWN_DIST + lateral.1,
        );

        const SPAWN_GAP: f32 = 2.0 * LANE_W;
        let dummy     = Vehicle::new(approach, route, base, 0.0);
        let spawn_prog = forward_progress(&dummy);
        if let Some(lane) = self.lanes.iter().find(|l| l.approach == approach) {
            let too_close = lane.incoming[sublane]
                .iter()
                .any(|v| (forward_progress(v) - spawn_prog).abs() < SPAWN_GAP);
            if too_close { return; }
        }

        let vehicle = Vehicle::new(approach, route, base, 120.0);
        if let Some(lane) = self.lanes.iter_mut().find(|l| l.approach == approach) {
            lane.incoming[sublane].push(vehicle);
        }
    }

    pub fn stats(&self) -> &Stats { &self.stats }

    fn next_random_approach(&mut self) -> Approach {
        if self.bag_index >= self.approach_bag.len() {
            self.refill_approach_bag();
        }
        let approach = self.approach_bag[self.bag_index];
        self.bag_index += 1;
        approach
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

    fn refill_approach_bag(&mut self) {
        self.approach_bag = vec![
            Approach::North,
            Approach::South,
            Approach::East,
            Approach::West,
        ];
        let mut i = self.approach_bag.len();
        while i > 1 {
            i -= 1;
            let j = (self.next_random_u32() as usize) % (i + 1);
            self.approach_bag.swap(i, j);
        }
        self.bag_index = 0;
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

fn distance_to_intersection_edge(position: (f32, f32), half_size: f32) -> f32 {
    position.0.abs().max(position.1.abs()) - half_size
}

/// Compute the expected [entry_time, exit_time] window for a vehicle.
///
/// `entry_time` = now + (distance to intersection edge) / speed
/// `exit_time`  = entry_time + (intersection diameter) / speed
///
/// Uses a minimum speed of 0.1 px/s to guard against division by zero
/// when the vehicle is temporarily stopped at the stop-line.
fn compute_reservation(
    position:     (f32, f32),
    heading:      (f32, f32),
    velocity:     f32,
    current_time: f32,
) -> (f32, f32) {
    let _ = heading; // heading not needed – distance is scalar
    let dist  = distance_to_intersection_edge(position, INTERSECTION_HALF).max(0.0);
    let speed = velocity.max(0.1);
    let entry_time = current_time + dist / speed;
    let exit_time  = entry_time + (3.0 * INTERSECTION_HALF) / speed;
    (entry_time, exit_time)
}