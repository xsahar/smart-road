// SDL2 app bootstrap: window creation, main loop, and high-level flow.
use sdl2::pixels::Color;
use std::time::{Duration, Instant};

use crate::input;
use crate::render;
use crate::sim::Simulation;

const WINDOW_WIDTH:  u32 = 900;
const WINDOW_HEIGHT: u32 = 900;

pub fn run() -> Result<(), String> {
    let sdl_context    = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    // TTF is optional - gracefully skip if the feature/library is unavailable.
    let ttf_context = sdl2::ttf::init().map_err(|e: _| e.to_string())?;

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

    let mut event_pump   = sdl_context.event_pump()?;
    let mut random_spawn = false;
    let mut last_spawn_time = Instant::now() - Duration::from_millis(1000);
    let spawn_cooldown   = Duration::from_millis(400);
    let mut last_frame   = Instant::now();

    let mut sim = Simulation::new();

    'running: loop {
        let now = Instant::now();
        let mut dt = now.duration_since(last_frame).as_secs_f32();
        last_frame = now;
        if dt > 0.05 { dt = 0.05; }

        let can_spawn   = last_spawn_time.elapsed() >= spawn_cooldown;
        let frame_input = input::collect(&mut event_pump, can_spawn);

        if frame_input.quit { break 'running; }

        if frame_input.toggle_random { random_spawn = !random_spawn; }

        if let Some(approach) = frame_input.spawn_approach {
            last_spawn_time = Instant::now();
            sim.request_spawn(approach);
        } else if random_spawn && can_spawn {
            last_spawn_time = Instant::now();
            sim.request_spawn_random();
        }

        sim.update(dt);

        canvas.set_draw_color(Color::RGB(20, 20, 20));
        canvas.clear();
        render::draw_intersection(&mut canvas, WINDOW_WIDTH, WINDOW_HEIGHT)?;
        render::draw_vehicles(&mut canvas, &sim.lanes, WINDOW_WIDTH, WINDOW_HEIGHT)?;
        canvas.present();
        std::thread::sleep(Duration::from_millis(16));
    }

    let summary = sim.stats().summary_line();
    println!("{}", summary);
    canvas.window_mut().set_title(&summary).map_err(|e| e.to_string())?;

    canvas.set_draw_color(Color::RGB(10, 10, 14));
    canvas.clear();
    render::draw_exit_screen(&mut canvas, WINDOW_WIDTH, WINDOW_HEIGHT, sim.stats(), &ttf_context)?;
    canvas.present();

    // Wait for keypress before closing so the stats screen stays visible.
    'exit: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. }    => break 'exit,
                sdl2::event::Event::KeyDown { .. } => break 'exit,
                _ => {}
            }
        }
        std::thread::sleep(Duration::from_millis(16));
    }

    Ok(())
}