// Rendering helpers for drawing the intersection and vehicles.
use sdl2::image::LoadTexture;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;
use std::path::Path;

use crate::sim::lane::Lane;
use crate::sim::LANE_W;
use crate::stats::Stats;

pub fn draw_intersection(
    canvas: &mut Canvas<Window>,
    width: u32,
    height: u32,
) -> Result<(), String> {
    const ROAD_TEXTURE_PATHS: [&str; 2] = [
        "assets/roads/intersection.png",
        "assets/roads/intersection.jpg",
    ];

    let texture_creator = canvas.texture_creator();
    for path in ROAD_TEXTURE_PATHS {
        if Path::new(path).exists() {
            if let Ok(texture) = texture_creator.load_texture(path) {
                let dst = Rect::new(0, 0, width, height);
                canvas.copy(&texture, None, dst)?;
                return Ok(());
            }
        }
    }

    let road_color = Color::RGB(55, 55, 55);
    let road_width: i32 = 220;
    let center_x = (width as i32) / 2;
    let center_y = (height as i32) / 2;

    // Draw road surface
    canvas.set_draw_color(road_color);
    let horizontal = Rect::new(0, center_y - road_width / 2, width, road_width as u32);
    let vertical = Rect::new(center_x - road_width / 2, 0, road_width as u32, height);
    canvas.fill_rect(horizontal)?;
    canvas.fill_rect(vertical)?;

    // Draw lane dividers based on simulation lane width
    canvas.set_draw_color(Color::RGB(200, 200, 100));
    let lane_w = LANE_W as i32;
    
    // Horizontal road lane lines
    for i in 1..3 {
        let y_top = center_y - road_width / 2 + i * lane_w;
        let y_bottom = center_y + road_width / 2 - i * lane_w;
        canvas.draw_line((0, y_top), (width as i32, y_top))?;
        canvas.draw_line((0, y_bottom), (width as i32, y_bottom))?;
    }
    
    // Vertical road lane lines
    for i in 1..3 {
        let x_left = center_x - road_width / 2 + i * lane_w;
        let x_right = center_x + road_width / 2 - i * lane_w;
        canvas.draw_line((x_left, 0), (x_left, height as i32))?;
        canvas.draw_line((x_right, 0), (x_right, height as i32))?;
    }

    Ok(())
}

pub fn draw_vehicles(
    canvas: &mut Canvas<Window>,
    lanes: &[Lane],
    width: u32,
    height: u32,
    car_texture: Option<&Texture>,
) -> Result<(), String> {
    let center_x = (width as i32) / 2;
    let center_y = (height as i32) / 2;
    let car_width: i32 = (LANE_W * 0.72) as i32;
    let car_height: i32 = (LANE_W * 1.28) as i32;
    let car_width = car_width.max(8);
    let car_height = car_height.max(12);

    for lane in lanes {
        for sublane in &lane.incoming {
            for vehicle in sublane {
                let x = center_x + vehicle.position.0 as i32 - car_width / 2;
                let y = center_y + vehicle.position.1 as i32 - car_height / 2;
                let rect = Rect::new(x, y, car_width as u32, car_height as u32);

                if let Some(texture) = car_texture {
                    let angle = heading_to_angle_deg(vehicle.heading);
                    let src = pick_car_sprite_src(texture);
                    canvas.copy_ex(texture, src, rect, angle, None, false, false)?;
                } else {
                    let color = match lane.approach {
                        crate::sim::vehicle::Approach::North => Color::RGB(230, 230, 230),
                        crate::sim::vehicle::Approach::South => Color::RGB(180, 220, 255),
                        crate::sim::vehicle::Approach::East => Color::RGB(255, 210, 170),
                        crate::sim::vehicle::Approach::West => Color::RGB(200, 255, 200),
                    };
                    canvas.set_draw_color(color);
                    canvas.fill_rect(rect)?;
                }
            }
        }
    }

    Ok(())
}

fn heading_to_angle_deg(heading: (f32, f32)) -> f64 {
    let a = (heading.0 as f64).atan2(-heading.1 as f64).to_degrees();
    (a + 360.0) % 360.0
}

fn pick_car_sprite_src(texture: &Texture) -> Option<Rect> {
    let q = texture.query();
    let w = q.width;
    let h = q.height;

    // Common atlas in this project: 2 columns x 3 rows (1024x1536).
    if w % 2 == 0 && h % 3 == 0 {
        return Some(Rect::new(0, 0, w / 2, h / 3));
    }

    // Fallback for 1x3 strips.
    if h % 3 == 0 {
        return Some(Rect::new(0, 0, w, h / 3));
    }

    None
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
