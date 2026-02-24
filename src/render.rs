// Rendering helpers: intersection, vehicles (with rotation), and exit screen.

use sdl2::image::LoadTexture;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureAccess};
use sdl2::video::Window;
use std::path::Path;

use crate::sim::lane::Lane;
use crate::sim::vehicle::Approach;
use crate::sim::LANE_W;
use crate::stats::Stats;

pub fn draw_intersection(
    canvas: &mut Canvas<Window>,
    width:  u32,
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
                canvas.copy(&texture, None, Rect::new(0, 0, width, height))?;
                return Ok(());
            }
        }
    }

    let road_color = Color::RGB(55, 55, 55);
    let road_width: i32 = 220;
    let cx = (width  as i32) / 2;
    let cy = (height as i32) / 2;

    canvas.set_draw_color(road_color);
    canvas.fill_rect(Rect::new(0,              cy - road_width / 2, width,            road_width as u32))?;
    canvas.fill_rect(Rect::new(cx - road_width / 2, 0,             road_width as u32, height))?;

    canvas.set_draw_color(Color::RGB(200, 200, 100));
    let lw = LANE_W as i32;
    for i in 1..3 {
        canvas.draw_line((0, cy - road_width / 2 + i * lw), (width as i32, cy - road_width / 2 + i * lw))?;
        canvas.draw_line((0, cy + road_width / 2 - i * lw), (width as i32, cy + road_width / 2 - i * lw))?;
        canvas.draw_line((cx - road_width / 2 + i * lw, 0), (cx - road_width / 2 + i * lw, height as i32))?;
        canvas.draw_line((cx + road_width / 2 - i * lw, 0), (cx + road_width / 2 - i * lw, height as i32))?;
    }

    Ok(())
}


pub fn draw_vehicles(
    canvas:      &mut Canvas<Window>,
    lanes:       &[Lane],
    width:       u32,
    height:      u32,
    car_texture: Option<&Texture>,
) -> Result<(), String> {
    let cx = (width  as i32) / 2;
    let cy = (height as i32) / 2;

    let car_w = ((LANE_W * 0.72) as i32).max(8)  as u32;
    let car_h = ((LANE_W * 1.28) as i32).max(12) as u32;

  
    let texture_creator = canvas.texture_creator();

    for lane in lanes {
        let fallback_color = approach_color(lane.approach);

        for sublane in &lane.incoming {
            for vehicle in sublane {
                let x = cx + vehicle.position.0 as i32 - (car_w as i32) / 2;
                let y = cy + vehicle.position.1 as i32 - (car_h as i32) / 2;
                let rect = Rect::new(x, y, car_w, car_h);

             
                let angle = heading_to_angle_deg(vehicle.heading);

                if let Some(texture) = car_texture {
                    // ── Real sprite: rotate using copy_ex ───────────────────
                    let src = pick_car_sprite_src(texture);
                    canvas.copy_ex(
                        texture,
                        src,
                        rect,
                        angle,
                        None,  
                        false,
                        false,
                    )?;
                } else {
                  
                    if let Ok(mut fallback_tex) = texture_creator.create_texture(
                        PixelFormatEnum::RGBA8888,
                        TextureAccess::Static,
                        car_w,
                        car_h,
                    ) {
                      
                        fallback_tex.set_color_mod(
                            fallback_color.r,
                            fallback_color.g,
                            fallback_color.b,
                        );

                        // Write solid pixels into the texture.
                        let pixel_count = (car_w * car_h) as usize;
                        let pixels: Vec<u8> = {
                            let (r, g, b) = (fallback_color.r, fallback_color.g, fallback_color.b);
                            (0..pixel_count).flat_map(|_| [r, g, b, 255u8]).collect()
                        };
                        let pitch = car_w * 4;
                        let _ = fallback_tex.update(None, &pixels, pitch as usize);

                        // copy_ex applies the rotation — same call as the real sprite.
                        canvas.copy_ex(
                            &fallback_tex,
                            None,
                            rect,
                            angle,
                            None,   
                            false,
                            false,
                        )?;
                    } else {
                        // Last resort: unrotated fill_rect (should never happen).
                        canvas.set_draw_color(fallback_color);
                        canvas.fill_rect(rect)?;
                    }
                }
            }
        }
    }

    Ok(())
}


fn heading_to_angle_deg(heading: (f32, f32)) -> f64 {
    // atan2(x, y) gives the angle from the +Y axis, clockwise.
    let deg = (heading.0 as f64).atan2(heading.1 as f64).to_degrees();
    (deg + 360.0) % 360.0
}

fn approach_color(approach: Approach) -> Color {
    match approach {
        Approach::North => Color::RGB(230, 230, 230),
        Approach::South => Color::RGB(180, 220, 255),
        Approach::East  => Color::RGB(255, 210, 170),
        Approach::West  => Color::RGB(200, 255, 200),
    }
}

fn pick_car_sprite_src(texture: &Texture) -> Option<Rect> {
    let q = texture.query();
    let w = q.width;
    let h = q.height;
    // Atlas: 2 columns × 3 rows
    if w % 2 == 0 && h % 3 == 0 {
        return Some(Rect::new(0, 0, w / 2, h / 3));
    }
    // 1×3 strip
    if h % 3 == 0 {
        return Some(Rect::new(0, 0, w, h / 3));
    }
    None
}

// ── Exit screen ──────────────────────────────────────────────────────────────

pub fn draw_exit_screen(
    canvas: &mut Canvas<Window>,
    width:  u32,
    height: u32,
    stats:  &Stats,
) -> Result<(), String> {
    let cx = (width  as i32) / 2;
    let cy = (height as i32) / 2;
    let panel_w: i32 = 420;
    let panel_h: i32 = 220;
    let panel = Rect::new(cx - panel_w / 2, cy - panel_h / 2, panel_w as u32, panel_h as u32);

    canvas.set_draw_color(Color::RGB(25, 25, 30));
    canvas.clear();
    canvas.set_draw_color(Color::RGB(60, 60, 70));
    canvas.fill_rect(panel)?;

    let bar_max: i32 = panel_w - 40;
    let passed      = stats.max_passed as f32;
    let close_calls = stats.close_calls as f32;
    let max_time    = stats.max_time;
    let scale       = (passed.max(close_calls).max(max_time)).max(1.0);

    let bars = [
        (passed,      Color::RGB(200, 220, 255)),
        (close_calls, Color::RGB(255, 210, 170)),
        (max_time,    Color::RGB(200, 255, 200)),
    ];

    for (i, (value, color)) in bars.iter().enumerate() {
        let w   = ((value / scale) * bar_max as f32) as i32;
        let y   = cy - 70 + (i as i32) * 60;
        let bar = Rect::new(cx - panel_w / 2 + 20, y, w.max(6) as u32, 16);
        canvas.set_draw_color(*color);
        canvas.fill_rect(bar)?;
    }

    Ok(())
}