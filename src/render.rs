// Rendering helpers for drawing the intersection and vehicles.
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::sim::lane::Lane;
use crate::stats::Stats;

pub fn draw_intersection(
    canvas: &mut Canvas<Window>,
    width: u32,
    height: u32,
) -> Result<(), String> {
    let road_color = Color::RGB(55, 55, 55);
    let road_width: i32 = 220;
    let center_x = (width as i32) / 2;
    let center_y = (height as i32) / 2;

    canvas.set_draw_color(road_color);
    let horizontal = Rect::new(0, center_y - road_width / 2, width, road_width as u32);
    let vertical = Rect::new(center_x - road_width / 2, 0, road_width as u32, height);
    canvas.fill_rect(horizontal)?;
    canvas.fill_rect(vertical)?;

    Ok(())
}

pub fn draw_vehicles(
    canvas: &mut Canvas<Window>,
    lanes: &[Lane],
    width: u32,
    height: u32,
) -> Result<(), String> {
    let center_x = (width as i32) / 2;
    let center_y = (height as i32) / 2;
    let car_width: i32 = 14;
    let car_height: i32 = 24;

    for lane in lanes {
        let color = match lane.approach {
            crate::sim::vehicle::Approach::North => Color::RGB(230, 230, 230),
            crate::sim::vehicle::Approach::South => Color::RGB(180, 220, 255),
            crate::sim::vehicle::Approach::East => Color::RGB(255, 210, 170),
            crate::sim::vehicle::Approach::West => Color::RGB(200, 255, 200),
        };
        canvas.set_draw_color(color);

        for vehicle in &lane.vehicles {
            let horizontal = vehicle.heading.0.abs() > vehicle.heading.1.abs();
            let (w, h) = if horizontal {
                (car_height, car_width)
            } else {
                (car_width, car_height)
            };
            let x = center_x + vehicle.position.0 as i32 - w / 2;
            let y = center_y + vehicle.position.1 as i32 - h / 2;
            let rect = Rect::new(x, y, w as u32, h as u32);
            canvas.fill_rect(rect)?;
        }
    }

    Ok(())
}

pub fn draw_exit_screen(
    canvas: &mut Canvas<Window>,
    width: u32,
    height: u32,
    stats: &Stats,
) -> Result<(), String> {
    let center_x = (width as i32) / 2;
    let center_y = (height as i32) / 2;
    let panel_w: i32 = 420;
    let panel_h: i32 = 220;
    let panel = Rect::new(center_x - panel_w / 2, center_y - panel_h / 2, panel_w as u32, panel_h as u32);

    canvas.set_draw_color(Color::RGB(25, 25, 30));
    canvas.clear();
    canvas.set_draw_color(Color::RGB(60, 60, 70));
    canvas.fill_rect(panel)?;

    let bar_max: i32 = panel_w - 40;
    let passed = stats.max_passed as f32;
    let close_calls = stats.close_calls as f32;
    let max_time = stats.max_time;
    let scale = (passed.max(close_calls).max(max_time)).max(1.0);

    let bars = [
        (passed, Color::RGB(200, 220, 255)),
        (close_calls, Color::RGB(255, 210, 170)),
        (max_time, Color::RGB(200, 255, 200)),
    ];

    for (i, (value, color)) in bars.iter().enumerate() {
        let width = ((value / scale) * (bar_max as f32)) as i32;
        let y = center_y - 70 + (i as i32) * 60;
        let rect = Rect::new(center_x - panel_w / 2 + 20, y, width.max(6) as u32, 16);
        canvas.set_draw_color(*color);
        canvas.fill_rect(rect)?;
    }

    Ok(())
}
