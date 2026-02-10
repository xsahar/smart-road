# Start here (beginner guide)

This guide is a step‑by‑step path to build the smart‑road project with Rust + SDL2.

## 1) What SDL2 is (quick mental model)
SDL2 is a Rust binding to a cross‑platform library that gives you:
- a window
- keyboard input
- a draw API (rectangles, textures, text) via the SDL2 renderer
- an event loop (poll events every frame)

## 2) First run (make sure the window appears)
From the project root, run:
- `cargo run`

You should see a window with a simple cross‑shaped road. If this works, your graphics/input stack is OK.

## 3) Project milestones (recommended order)
### Milestone A — Data model (no rendering yet)
Create basic structs and logic:
- Vehicle (route, position, velocity level, time/distance)
- Lane queues (one per approach)
- Safety distance checks

Goal: a pure simulation step that updates vehicles without collisions.

### Milestone B — Spawning + input
- Arrow keys spawn vehicles from the correct approach
- `R` toggles random spawning
- Rate limit repeated spawns (cooldown already in the scaffold)

Goal: you can spawn vehicles and see them appear in the right lane.

### Milestone C — Movement + turning
- Move vehicles along their lane
- When turning, rotate the vehicle texture while it turns

Goal: vehicles move through the intersection without clipping.

### Milestone D — Stats + exit
- Track max/min velocity, max/min time, max passed, close calls
- On `Esc`, show a stats window (can be a simple second window or a full‑screen stats overlay)

Goal: satisfy all checks in [Audit.md](../Audit.md).

## 4) Where to get images (optional at first)
You can start with rectangles (built‑in shapes) and add images later.
When ready, get sprites from:
- https://limezu.itch.io/
- http://finalbossblues.com/timefantasy/free-graphics/
- https://mobilegamegraphics.com/product-category/all_products/freestuff/
- https://www.spriters-resource.com/

Add an `assets/` folder and load textures with SDL2 image support (we can wire this later).

## 5) Suggested file layout (as you grow)
- src/main.rs (entry + loop)
- src/sim/ (simulation logic)
- src/render/ (drawing)
- src/input/ (key handling)
- assets/ (images)

Don’t build everything at once. Start small and keep it working.

## 6) Key rules to keep in mind
- Each lane has fixed route: `r`, `s`, `l`
- Vehicles can’t change lanes
- At least 3 velocity levels
- Safe distance must be strictly positive
- Vehicles must rotate when turning

## 7) How to read the spec
Use:
- [README.md](../README.md) for requirements
- [Audit.md](../Audit.md) for what will be tested

If you want, ask for the next step and I’ll implement it with you.
