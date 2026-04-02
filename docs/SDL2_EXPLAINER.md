# SDL2 beginner explainer (Rust + Windows)

This explains what each install step does and how the SDL2 window setup works in this project.

## What each install step does

### Rust + Cargo
- Rust is the language.
- Cargo is the build tool and package manager. It downloads crates (libraries) and builds your project.

### vcpkg
- vcpkg is a C/C++ package manager for Windows.
- SDL2 is a C library, so Rust needs the compiled C library files.
- vcpkg downloads, builds, and installs SDL2 for your system.

### SDL2 library
- SDL2 provides the real window, keyboard input, and drawing functions.
- The Rust crate `sdl2` is just a wrapper that calls into the SDL2 C library.

### Environment variables
- `VCPKG_ROOT` tells the Rust build where your vcpkg is installed.
- `VCPKG_DEFAULT_TRIPLET` tells vcpkg which build type to use (here: 64-bit, static MD runtime).

### Cargo.toml dependency
- `sdl2 = { version = "0.38", features = ["use-vcpkg", "static-link"] }`
- `use-vcpkg` makes the crate search vcpkg for SDL2.
- `static-link` matches the installed triplet and links the static libraries correctly.

## How the SDL2 window setup works (code overview)

In [src/main.rs](../src/main.rs), the setup has these steps:

1) Initialize SDL2
- `let sdl_context = sdl2::init()?;`
- This starts SDL2 so you can create windows, use input, etc.

2) Create the video subsystem
- `let video_subsystem = sdl_context.video()?;`
- SDL2 is split into subsystems. We only need video for a window.

3) Create a window
- `video_subsystem.window("smart-road", 900, 900).build()?;`
- This opens a real OS window titled "smart-road".

4) Create a canvas (renderer)
- `let mut canvas = window.into_canvas().present_vsync().build()?;`
- The canvas is the drawing target. You draw rectangles, textures, and text here.
- `present_vsync` syncs drawing to the monitor refresh rate.

5) Create an event pump
- `let mut event_pump = sdl_context.event_pump()?;`
- This reads keyboard and window events each frame.

6) Game loop
- `loop { ... }`
- Every frame:
  - Read events (keyboard, quit)
  - Update logic (spawn timers)
  - Draw the scene (roads, cars)
  - `canvas.present()` shows the new frame

## Why the window opens
When `cargo run` starts your program, SDL2 creates a native window. It is not a browser. It is your actual application window.

## Next learning steps
- Add a struct for `Vehicle` and spawn one with Arrow Up.
- Move the vehicle every frame.
- Then add turning logic and safe distance.
