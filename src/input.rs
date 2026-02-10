// Input handling: converts SDL events into frame-level actions.
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::EventPump;

pub struct FrameInput {
    pub quit: bool,
    pub toggle_random: bool,
    pub spawn_requested: bool,
}

pub fn collect(event_pump: &mut EventPump, can_spawn: bool) -> FrameInput {
    let mut quit = false;
    let mut toggle_random = false;
    let mut spawn_requested = false;

    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. } => quit = true,
            Event::KeyDown {
                keycode: Some(Keycode::Escape),
                repeat: false,
                ..
            } => quit = true,
            Event::KeyDown {
                keycode: Some(Keycode::R),
                repeat: false,
                ..
            } => toggle_random = true,
            Event::KeyDown {
                keycode: Some(Keycode::Up | Keycode::Down | Keycode::Left | Keycode::Right),
                repeat: false,
                ..
            } => {
                if can_spawn {
                    spawn_requested = true;
                }
            }
            _ => {}
        }
    }

    FrameInput {
        quit,
        toggle_random,
        spawn_requested,
    }
}
