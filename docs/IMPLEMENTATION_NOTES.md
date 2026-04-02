# Implementation notes

This document captures a step-by-step, line-by-line explanation of the current simulation scaffolding and how it is wired into the SDL loop.

## Simulation module

File: src/sim/mod.rs

- Line 1: File header comment describing the module intent.
- Line 2: `pub mod lane;` exposes the lane submodule.
- Line 3: `pub mod vehicle;` exposes the vehicle submodule.
- Line 5: `use lane::Lane;` brings `Lane` into scope.
- Line 6: `use vehicle::{Approach, Route, Vehicle};` brings vehicle types into scope.
- Line 8: `pub struct Simulation {` starts the sim state definition.
- Line 9: `pub lanes: Vec<Lane>,` stores all lanes for the intersection.
- Line 10: `pending_spawns: Vec<Approach>,` queues spawn requests with directions.
- Line 11: `rng_state: u32,` stores a simple PRNG state for random spawns.
- Line 14: `pub fn new() -> Self {` constructs a new sim state.
- Line 15-21: Initializes four lanes, one per approach.
- Line 22: `pending_spawns: Vec::new(),` starts the spawn queue empty.
- Line 23: `rng_state: 0x1234ABCD,` seeds the PRNG.
- Line 27: `pub fn request_spawn(&mut self, approach: Approach)` queues a spawn for the given approach.
- Line 31: `pub fn request_spawn_random(&mut self)` picks an approach using the PRNG and queues it.
- Line 36: `pub fn update(&mut self)` is the per-frame sim step.
- Line 37-39: Returns early if there is nothing to spawn.
- Line 41-44: Swaps the pending queue into a local buffer to drain it safely.
- Line 45-47: Spawns a vehicle for each queued approach.
- Line 50: `fn spawn_vehicle(&mut self, approach: Approach)` builds and inserts a new vehicle.
- Line 51: `let route = Route::Straight;` sets a default route.
- Line 52-57: Chooses a starting position based on the approach.
- Line 58: Constructs a `Vehicle` with position and velocity.
- Line 59-61: Finds the matching lane and pushes the new vehicle.
- Line 64: `fn next_random_approach(&mut self)` updates the PRNG and maps it to an approach.
- Line 65-67: LCG step using wrapping arithmetic.
- Line 68-72: Maps the PRNG output to North/South/East/West.

## Lane model

File: src/sim/lane.rs

- Line 1: File header comment.
- Line 2: Imports `Approach` and `Vehicle` types.
- Line 4: `pub struct Lane {` starts the lane definition.
- Line 5: `pub approach: Approach,` records which approach this lane serves.
- Line 6: `pub vehicles: Vec<Vehicle>,` keeps vehicles currently in the lane.
- Line 9: `impl Lane {` begins the implementation.
- Line 10: `pub fn new(approach: Approach) -> Self` constructs a lane.
- Line 11-14: Initializes the lane with an empty vehicle list.

## Vehicle model

File: src/sim/vehicle.rs

- Line 1: File header comment.
- Line 2-7: `Route` enum declares turn choices.
- Line 9-15: `Approach` enum declares entry directions.
- Line 17-24: `Vehicle` struct defines approach, route, position, velocity, and elapsed time.
- Line 26-37: `Vehicle::new` initializes a vehicle with `elapsed` set to 0.0.

## App loop integration

File: src/app.rs

- Line 7: Imports the `Simulation` type.
- Line 24: Creates the sim with `Simulation::new()`.
- Line 33-42: Input is collected, and quit/toggle actions are handled.
- Line 44-50: Spawns are requested when a key is pressed or random mode is active.
- Line 52: `sim.update()` runs the per-frame sim update.
- Line 55-60: Clears and redraws the frame.

## Input handling

File: src/input.rs

- Line 1-4: Imports SDL event types and the event pump.
- Line 6: Imports `Approach` so we can map arrow keys to directions.
- Line 8-12: `FrameInput` stores per-frame actions, including an optional spawn direction.
- Line 14: `collect` starts reading SDL events.
- Line 15-17: Initializes the per-frame flags.
- Line 19-50: Maps SDL events to actions.
- Line 20-22: Window close sets `quit`.
- Line 23-27: Escape key sets `quit`.
- Line 28-32: R toggles random spawning.
- Line 33-50: Arrow keys set `spawn_approach` to North, South, West, or East when spawning is allowed.
- Line 54-58: Returns the populated `FrameInput`.

## Program entry

File: src/main.rs

- Line 2-5: Declares module roots.
- Line 7: `main` calls `app::run()` to start the SDL loop.
