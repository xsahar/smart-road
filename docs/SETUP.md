# Setup (SDL2)

This is the exact setup flow used here.

## Option A — Create a fresh project folder
1) Open a terminal in the parent directory.
2) Run:
   - `cargo new smart-road --bin`
3) Enter the new folder:
   - `cd smart-road`
4) Open Cargo.toml and add:
   - `sdl2 = { version = "0.38", features = ["use-vcpkg", "static-link"] }` under `[dependencies]`
5) Run:
   - `cargo run`

## Option B — Initialize inside an existing folder
Use this if you already created the folder (for example, `smart-road`).
1) Open a terminal inside that folder.
2) Run:
   - `cargo init --bin .`
3) Add to Cargo.toml under `[dependencies]`:
   - `sdl2 = { version = "0.38", features = ["use-vcpkg", "static-link"] }`
4) Run:
   - `cargo run`

## Windows setup (what was installed)
This is the full, working Windows setup used here.

1) Install vcpkg:
   - `git clone https://github.com/microsoft/vcpkg C:\vcpkg`
   - `cd C:\vcpkg`
   - `\bootstrap-vcpkg.bat`

2) Install SDL2 (static MD triplet):
   - `\vcpkg.exe install sdl2:x64-windows-static-md`

3) Set environment variables (PowerShell):
   - `setx VCPKG_ROOT "C:\vcpkg"`
   - `setx VCPKG_DEFAULT_TRIPLET "x64-windows-static-md"`

4) (Optional) Persist for this repo via Cargo config:
   - Create `.cargo/config.toml` with:
     - `VCPKG_ROOT = "C:/vcpkg"`
     - `VCPKG_DEFAULT_TRIPLET = "x64-windows-static-md"`

5) Build and run:
   - `cargo run`

## Notes
- `cargo run` creates Cargo.lock and builds the project.
- If you accidentally run `cargo new smart-road --bin` *inside* `smart-road`, it creates a nested `smart-road/smart-road`.

## Linux setup (quick)
1) Install SDL2 dev packages:
   - `sudo apt update`
   - `sudo apt install libsdl2-dev`
2) Build and run:
   - `cargo run`

## References
- https://docs.rs/sdl2 — API documentation and usage examples.
- https://github.com/Rust-SDL2/rust-sdl2 (examples folder) — runnable sample projects and patterns.
