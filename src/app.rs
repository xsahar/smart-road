// SDL2 app bootstrap: window creation, main loop, and high-level flow.
use sdl2::image::{InitFlag, LoadTexture};
use sdl2::pixels::Color;
use std::time::{Duration, Instant};
use std::path::Path;

use crate::input;
use crate::render;
use crate::sim::Simulation;

const WINDOW_WIDTH: u32 = 900;
const WINDOW_HEIGHT: u32 = 900;

pub fn run() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let _image_ctx = sdl2::image::init(InitFlag::PNG | InitFlag::JPG).ok();
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

    let texture_creator = canvas.texture_creator();
    let car_texture = load_car_texture(&texture_creator);

    let mut event_pump = sdl_context.event_pump()?;
    let mut random_spawn = false;
    let mut last_spawn_time = Instant::now() - Duration::from_millis(1000);
    let spawn_cooldown = Duration::from_millis(400);
    let mut last_frame = Instant::now();

    let mut sim = Simulation::new();

    'running: loop {
        let now = Instant::now();
        let mut dt = now.duration_since(last_frame).as_secs_f32();
        last_frame = now;
        if dt > 0.05 {
            dt = 0.05;
        }

        let can_spawn = last_spawn_time.elapsed() >= spawn_cooldown;
        let frame_input = input::collect(&mut event_pump, can_spawn);

        if frame_input.quit {
            break 'running;
        }

        if frame_input.toggle_random {
            random_spawn = !random_spawn;
        }

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
        render::draw_vehicles(
            &mut canvas,
            &sim.lanes,
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            car_texture.as_ref(),
        )?;
        canvas.present();
        std::thread::sleep(Duration::from_millis(16));
    }

    let summary = sim.stats().summary_line();
    println!("{}", summary);
    canvas
        .window_mut()
        .set_title(&summary)
        .map_err(|e| e.to_string())?;
    render::draw_exit_screen(&mut canvas, WINDOW_WIDTH, WINDOW_HEIGHT, sim.stats())?;
    canvas.present();
    std::thread::sleep(Duration::from_millis(1500));

    Ok(())
}

fn load_car_texture(
    texture_creator: &sdl2::render::TextureCreator<sdl2::video::WindowContext>,
) -> Option<sdl2::render::Texture<'_>> {
    const CAR_TEXTURE_PATHS: [&str; 2] = [
        "assets/cars/car.png",
        "assets/cars/car.jpg",
    ];

    for path in CAR_TEXTURE_PATHS {
        if Path::new(path).exists() {
            if let Ok(texture) = texture_creator.load_texture(path) {
                return Some(texture);
            }
        }
    }

    None
}
