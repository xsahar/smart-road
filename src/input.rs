// Input handling: converts SDL events into frame-level actions.
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::EventPump;

use crate::sim::vehicle::Approach;

pub struct FrameInput {
    pub quit: bool,
    pub toggle_random: bool,
    pub spawn_approach: Option<Approach>,
}

pub fn collect(event_pump: &mut EventPump, can_spawn: bool) -> FrameInput {
    let mut quit = false;
    let mut toggle_random = false;
    let mut spawn_approach = None;

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
                keycode: Some(Keycode::Up),
                repeat: false,
                ..
            } => {
                if can_spawn {
                    spawn_approach = Some(Approach::North);
                }
            }
            Event::KeyDown {
                keycode: Some(Keycode::Down),
                repeat: false,
                ..
            } => {
                if can_spawn {
                    spawn_approach = Some(Approach::South);
                }
            }
            Event::KeyDown {
                keycode: Some(Keycode::Left),
                repeat: false,
                ..
            } => {
                if can_spawn {
                    spawn_approach = Some(Approach::West);
                }
            }
            Event::KeyDown {
                keycode: Some(Keycode::Right),
                repeat: false,
                ..
            } => {
                if can_spawn {
                    spawn_approach = Some(Approach::East);
                }
            }
            _ => {}
        }
    }

    FrameInput {
        quit,
        toggle_random,
        spawn_approach,
    }
}
