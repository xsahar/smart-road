// SDL2 app bootstrap: window creation, main loop, and high-level flow.
use sdl2::pixels::Color;
use std::time::{Duration, Instant};

use crate::input;
use crate::render;
use crate::sim::Simulation;

const WINDOW_WIDTH: u32 = 900;
const WINDOW_HEIGHT: u32 = 900;

pub fn run() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("smart-road", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;

    let mut event_pump = sdl_context.event_pump()?;
    let mut random_spawn = false;
    let mut last_spawn_time = Instant::now() - Duration::from_millis(1000);
    let spawn_cooldown = Duration::from_millis(400);

    let mut sim = Simulation::new();

    'running: loop {
        let can_spawn = last_spawn_time.elapsed() >= spawn_cooldown;
        let frame_input = input::collect(&mut event_pump, can_spawn);

        if frame_input.quit {
            break 'running;
        }

        if frame_input.toggle_random {
            random_spawn = !random_spawn;
        }

        let mut spawn_request = frame_input.spawn_requested;
        if random_spawn && can_spawn {
            spawn_request = true;
        }

        if spawn_request {
            last_spawn_time = Instant::now();
            sim.request_spawn();
        }

        sim.update();

        canvas.set_draw_color(Color::RGB(20, 20, 20));
        canvas.clear();
        render::draw_intersection(&mut canvas, WINDOW_WIDTH, WINDOW_HEIGHT)?;
        canvas.present();
        std::thread::sleep(Duration::from_millis(16));
    }

    Ok(())
}
