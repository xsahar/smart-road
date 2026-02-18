// Rendering: intersection with exact 6-lane layout, animated vehicles, stats screen.
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::ttf::Sdl2TtfContext;

use crate::sim::lane::Lane;
use crate::sim::LANE_W;
use crate::sim::vehicle::Approach;
use crate::stats::Stats;


const ROAD_HALF: i32 = (3.0 * LANE_W) as i32; // = 90px  (total road = 180px)

// ── Colours ──────────────────────────────────────────────────────────────────
const C_GRASS:     Color = Color::RGB(34,  90,  34);
const C_ASPHALT:   Color = Color::RGB(50,  50,  53);
const C_CURB:      Color = Color::RGB(22,  22,  24);
const C_DIVIDER:   Color = Color::RGB(240, 210,  40);  // yellow centre divider
const C_DASH:      Color = Color::RGB(210, 210, 210);  // white lane dashes
const C_CROSSWALK: Color = Color::RGB(195, 195, 195);

fn car_color(approach: Approach) -> Color {
    match approach {
        Approach::North => Color::RGB(210,  55,  55),
        Approach::South => Color::RGB( 55, 120, 215),
        Approach::East  => Color::RGB(225, 160,  35),
        Approach::West  => Color::RGB( 45, 185,  85),
    }
}

pub fn draw_intersection(canvas: &mut Canvas<Window>, width: u32, height: u32) -> Result<(), String> {
    let w  = width  as i32;
    let h  = height as i32;
    let cx = w / 2;
    let cy = h / 2;
    let rh = ROAD_HALF;

    // Grass
    canvas.set_draw_color(C_GRASS);
    canvas.fill_rect(Rect::new(0, 0, width, height))?;

    // Asphalt
    canvas.set_draw_color(C_ASPHALT);
    canvas.fill_rect(Rect::new(0,       cy - rh, width,        (2 * rh) as u32))?; // horizontal
    canvas.fill_rect(Rect::new(cx - rh, 0,       (2 * rh) as u32, height))?;       // vertical

    // Curb edges (2px dark strip)
    canvas.set_draw_color(C_CURB);
    let c = 2i32;
    canvas.fill_rect(Rect::new(0,       cy - rh,       width,        c as u32))?;
    canvas.fill_rect(Rect::new(0,       cy + rh - c,   width,        c as u32))?;
    canvas.fill_rect(Rect::new(cx - rh, 0,             c as u32,     height))?;
    canvas.fill_rect(Rect::new(cx + rh - c, 0,         c as u32,     height))?;

    // Yellow centre divider - runs BETWEEN the 3 incoming and 3 outgoing lanes.
    // For the vertical road the divider is a vertical stripe at x = cx (outside intersection).
    // For the horizontal road it's a horizontal stripe at y = cy.
    let dw = 3i32;
    canvas.set_draw_color(C_DIVIDER);
    // vertical road centre divider (outside intersection box)
    canvas.fill_rect(Rect::new(cx - dw/2, 0,      dw as u32, (cy - rh) as u32))?;
    canvas.fill_rect(Rect::new(cx - dw/2, cy + rh, dw as u32, (h - cy - rh) as u32))?;
    // horizontal road centre divider (outside intersection box)
    canvas.fill_rect(Rect::new(0,      cy - dw/2, (cx - rh) as u32, dw as u32))?;
    canvas.fill_rect(Rect::new(cx + rh, cy - dw/2, (w - cx - rh) as u32, dw as u32))?;


    let lw = LANE_W as i32;
    let dash_len  = 18i32;
    let dash_gap  = 12i32;
    let dash_step = dash_len + dash_gap;
    let dline = 2i32;

    canvas.set_draw_color(C_DASH);

    // Vertical road: horizontal dashes at fixed x offsets
    for side in [-1i32, 1i32] {
        for lane_off in [lw, 2 * lw] {
            let x = cx + side * lane_off;
            let mut y = 0i32;
            while y < h {
                if y + dash_len < cy - rh || y > cy + rh {
                    canvas.fill_rect(Rect::new(x - dline/2, y, dline as u32, dash_len as u32))?;
                }
                y += dash_step;
            }
        }
    }

    // Horizontal road: vertical dashes at fixed y offsets
    for side in [-1i32, 1i32] {
        for lane_off in [lw, 2 * lw] {
            let y = cy + side * lane_off;
            let mut x = 0i32;
            while x < w {
                if x + dash_len < cx - rh || x > cx + rh {
                    canvas.fill_rect(Rect::new(x, y - dline/2, dash_len as u32, dline as u32))?;
                }
                x += dash_step;
            }
        }
    }

    // Crosswalk stripes just outside the intersection box
    canvas.set_draw_color(C_CROSSWALK);
    let sw  = 7i32;  // stripe width
    let sg  = 5i32;  // stripe gap
    let ss  = sw + sg;
    let cd  = 16i32; // crosswalk depth

    // Top crosswalk (spans the vertical road)
    let top_y = cy - rh - cd;
    let mut x = cx - rh;
    while x < cx + rh {
        canvas.fill_rect(Rect::new(x, top_y, sw as u32, cd as u32))?;
        x += ss;
    }
    // Bottom crosswalk
    let bot_y = cy + rh;
    let mut x = cx - rh;
    while x < cx + rh {
        canvas.fill_rect(Rect::new(x, bot_y, sw as u32, cd as u32))?;
        x += ss;
    }
    // Left crosswalk
    let left_x = cx - rh - cd;
    let mut y = cy - rh;
    while y < cy + rh {
        canvas.fill_rect(Rect::new(left_x, y, cd as u32, sw as u32))?;
        y += ss;
    }
    // Right crosswalk
    let right_x = cx + rh;
    let mut y = cy - rh;
    while y < cy + rh {
        canvas.fill_rect(Rect::new(right_x, y, cd as u32, sw as u32))?;
        y += ss;
    }

    // Route arrows in each incoming lane (outside intersection)
    draw_route_arrows(canvas, cx, cy, rh)?;

    Ok(())
}

/// Draw route indicator arrows in each of the 3 incoming lanes per approach,
/// just outside the intersection box (inside the road but before the crosswalk).
fn draw_route_arrows(canvas: &mut Canvas<Window>, cx: i32, cy: i32, rh: i32) -> Result<(), String> {
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    let lw = LANE_W as i32;

    // Lanes are LEFT of heading (right-hand traffic, right side of road).
    // left-of-heading offsets: r=2.5L (road edge), s=1.5L, l=0.5L (near divider)
    let arrow_dist = rh + 40;

    // North approach (heading DOWN, left-of-heading = screen RIGHT)
    //   r at cx+2.5L, s at cx+1.5L, l at cx+0.5L
    let ay = cy - arrow_dist;
    for (i, off) in [5*lw/2, 3*lw/2, lw/2].iter().enumerate() {
        let ax = cx + off;
        let dir = match i { 0 => ArrowDir::Left, 1 => ArrowDir::Down, _ => ArrowDir::Right };
        draw_arrow(canvas, ax, ay, dir)?;
    }

    // South approach (heading UP, left-of-heading = screen LEFT)
    //   r at cx-2.5L, s at cx-1.5L, l at cx-0.5L
    let ay = cy + arrow_dist;
    for (i, off) in [5*lw/2, 3*lw/2, lw/2].iter().enumerate() {
        let ax = cx - off;
        let dir = match i { 0 => ArrowDir::Right, 1 => ArrowDir::Up, _ => ArrowDir::Left };
        draw_arrow(canvas, ax, ay, dir)?;
    }

    // East approach (heading LEFT, left-of-heading = screen DOWN)
    //   r at cy+2.5L, s at cy+1.5L, l at cy+0.5L
    let ax = cx + arrow_dist;
    for (i, off) in [5*lw/2, 3*lw/2, lw/2].iter().enumerate() {
        let ay = cy + off;
        let dir = match i { 0 => ArrowDir::Up, 1 => ArrowDir::Left, _ => ArrowDir::Down };
        draw_arrow(canvas, ax, ay, dir)?;
    }

    // West approach (heading RIGHT, left-of-heading = screen UP)
    //   r at cy-2.5L, s at cy-1.5L, l at cy-0.5L
    let ax = cx - arrow_dist;
    for (i, off) in [5*lw/2, 3*lw/2, lw/2].iter().enumerate() {
        let ay = cy - off;
        let dir = match i { 0 => ArrowDir::Down, 1 => ArrowDir::Right, _ => ArrowDir::Up };
        draw_arrow(canvas, ax, ay, dir)?;
    }

    Ok(())
}

#[derive(Clone, Copy)]
enum ArrowDir { Up, Down, Left, Right }

fn draw_arrow(canvas: &mut Canvas<Window>, x: i32, y: i32, dir: ArrowDir) -> Result<(), String> {
    let s = 3i32; // scale
    match dir {
        ArrowDir::Up => {
            // shaft
            canvas.fill_rect(Rect::new(x - s/2, y, s as u32, (s*3) as u32))?;
            // head (triangle pointing up)
            for i in 0..=2i32 {
                let hw = s/2 + i * s / 2;
                canvas.fill_rect(Rect::new(x - hw, y - i * s / 2 - s, (hw*2 + 1) as u32, (s/2 + 1) as u32))?;
            }
        }
        ArrowDir::Down => {
            canvas.fill_rect(Rect::new(x - s/2, y - s*3, s as u32, (s*3) as u32))?;
            for i in 0..=2i32 {
                let hw = s/2 + i * s / 2;
                canvas.fill_rect(Rect::new(x - hw, y + i * s / 2, (hw*2 + 1) as u32, (s/2 + 1) as u32))?;
            }
        }
        ArrowDir::Left => {
            canvas.fill_rect(Rect::new(x, y - s/2, (s*3) as u32, s as u32))?;
            for i in 0..=2i32 {
                let hh = s/2 + i * s / 2;
                canvas.fill_rect(Rect::new(x - i * s / 2 - s, y - hh, (s/2 + 1) as u32, (hh*2 + 1) as u32))?;
            }
        }
        ArrowDir::Right => {
            canvas.fill_rect(Rect::new(x - s*3, y - s/2, (s*3) as u32, s as u32))?;
            for i in 0..=2i32 {
                let hh = s/2 + i * s / 2;
                canvas.fill_rect(Rect::new(x + i * s / 2, y - hh, (s/2 + 1) as u32, (hh*2 + 1) as u32))?;
            }
        }
    }
    Ok(())
}


pub fn draw_vehicles(canvas: &mut Canvas<Window>, lanes: &[Lane], width: u32, height: u32) -> Result<(), String> {
    let cx = (width  as i32) / 2;
    let cy = (height as i32) / 2;

    for lane in lanes {
        for sublane in &lane.incoming {
            for vehicle in sublane {
                let sx = cx + vehicle.position.0 as i32;
                let sy = cy + vehicle.position.1 as i32;
                let angle = heading_to_angle_deg(vehicle.heading);
                draw_car(canvas, sx, sy, angle, car_color(vehicle.approach))?;
            }
        }
    }
    Ok(())
}

fn heading_to_angle_deg(heading: (f32, f32)) -> f64 {
    // atan2(x, -y): heading (0,-1)=up→0°, (1,0)=right→90°, (0,1)=down→180°, (-1,0)=left→270°
    let a = (heading.0 as f64).atan2(-heading.1 as f64).to_degrees();
    (a + 360.0) % 360.0
}

/// Pixel-art car drawn as rotated axis-aligned rectangles.
fn draw_car(canvas: &mut Canvas<Window>, cx: i32, cy: i32, angle_deg: f64, body: Color) -> Result<(), String> {
    const CW: i32 = 13; // car width
    const CH: i32 = 22; // car height (forward axis)

    let rad = angle_deg.to_radians();
    let cos = rad.cos() as f32;
    let sin = rad.sin() as f32;

    // Rotate local (lx, ly) offset to screen space and draw a rect centred there.
    let blit = |canvas: &mut Canvas<Window>, lx: f32, ly: f32, w: i32, h: i32, col: Color| -> Result<(), String> {
        canvas.set_draw_color(col);
        let rx = cx + (lx * cos - ly * sin) as i32;
        let ry = cy + (lx * sin + ly * cos) as i32;
        canvas.fill_rect(Rect::new(rx - w/2, ry - h/2, w as u32, h as u32))
    };

    // Body
    blit(canvas, 0.0, 0.0, CW, CH, body)?;
    // Windshield (front = negative local-y = top of car)
    blit(canvas, 0.0, -(CH/2 - 4) as f32, CW - 4, 5, Color::RGB(155, 205, 225))?;
    // Rear window
    blit(canvas, 0.0,  (CH/2 - 5) as f32, CW - 4, 4, Color::RGB(100, 140, 165))?;
    // Headlights
    let hly = -(CH/2) as f32 + 1.5;
    blit(canvas, -(CW/2 - 2) as f32, hly, 3, 3, Color::RGB(255, 255, 170))?;
    blit(canvas,  (CW/2 - 2) as f32, hly, 3, 3, Color::RGB(255, 255, 170))?;
    // Tail lights
    let tly = (CH/2) as f32 - 1.5;
    blit(canvas, -(CW/2 - 2) as f32, tly, 3, 3, Color::RGB(195, 30, 30))?;
    blit(canvas,  (CW/2 - 2) as f32, tly, 3, 3, Color::RGB(195, 30, 30))?;
    // Wheels (dark strips on sides)
    for &wy in &[-4.0f32, 4.0] {
        blit(canvas, -(CW/2) as f32, wy, 3, 7, Color::RGB(18, 18, 18))?;
        blit(canvas,  (CW/2) as f32, wy, 3, 7, Color::RGB(18, 18, 18))?;
    }
    Ok(())
}

pub fn draw_exit_screen(
    canvas: &mut Canvas<Window>,
    width: u32,
    height: u32,
    stats: &Stats,
    ttf: &Sdl2TtfContext,
) -> Result<(), String> {
    let w  = width  as i32;
    let h  = height as i32;
    let cx = w / 2;
    let cy = h / 2;

    canvas.set_draw_color(Color::RGB(10, 10, 14));
    canvas.fill_rect(Rect::new(0, 0, width, height))?;

    let pw = 500i32;
    let ph = 420i32;
    let px = cx - pw / 2;
    let py = cy - ph / 2;

    canvas.set_draw_color(Color::RGB(22, 26, 38));
    canvas.fill_rect(Rect::new(px, py, pw as u32, ph as u32))?;

    // Border
    canvas.set_draw_color(Color::RGB(60, 75, 120));
    canvas.draw_rect(Rect::new(px - 1, py - 1, (pw + 2) as u32, (ph + 2) as u32))?;
    canvas.draw_rect(Rect::new(px - 2, py - 2, (pw + 4) as u32, (ph + 4) as u32))?;

    // Title bar
    canvas.set_draw_color(Color::RGB(35, 45, 85));
    canvas.fill_rect(Rect::new(px, py, pw as u32, 44))?;

    let tc = canvas.texture_creator();
    let min_vel  = if stats.min_velocity.is_finite() { stats.min_velocity } else { 0.0 };
    let min_time = if stats.min_time.is_finite() { stats.min_time } else { 0.0 };

    let rows: &[(&str, String, Color)] = &[
        ("Vehicles passed",    format!("{}",         stats.max_passed),   Color::RGB(175, 215, 255)),
        ("Max velocity",       format!("{:.1} px/s", stats.max_velocity), Color::RGB(115, 225, 145)),
        ("Min velocity",       format!("{:.1} px/s", min_vel),            Color::RGB(115, 225, 145)),
        ("Max transit time",   format!("{:.2} s",    stats.max_time),     Color::RGB(255, 205, 115)),
        ("Min transit time",   format!("{:.2} s",    min_time),           Color::RGB(255, 205, 115)),
        ("Close calls",        format!("{}",         stats.close_calls),  Color::RGB(255, 115, 115)),
    ];

    let font_result = match find_system_font() {
        Some(path) => ttf.load_font(path, 18).map_err(|e| e.to_string()),
        None => Err("no font found".to_string()),
    };
    match font_result {
        Ok(font) => {
            render_text(canvas, &tc, &font, "SIMULATION RESULTS", Color::RGB(215, 225, 255), cx, py + 22)?;

            let row_h    = 52i32;
            let bar_zone = pw - 210;
            let max_num  = (stats.max_passed as f32)
                .max(stats.max_velocity)
                .max(stats.max_time * 10.0)
                .max(stats.close_calls as f32)
                .max(1.0);

            for (i, (label, value, color)) in rows.iter().enumerate() {
                let ry = py + 54 + i as i32 * row_h;

                // Label
                render_text(canvas, &tc, &font, label, Color::RGB(175, 175, 200), px + 14, ry + row_h / 2)?;

                // Bar bg
                canvas.set_draw_color(Color::RGB(32, 36, 52));
                canvas.fill_rect(Rect::new(px + 205, ry + 11, bar_zone as u32, (row_h - 26) as u32))?;

                // Bar fill
                let num: f32 = value.split_whitespace().next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
                let bw = ((num / max_num) * bar_zone as f32) as i32;
                canvas.set_draw_color(*color);
                canvas.fill_rect(Rect::new(px + 205, ry + 11, bw.max(4) as u32, (row_h - 26) as u32))?;

                // Value label
                render_text(canvas, &tc, &font, value, Color::RGB(240, 240, 240), px + 205 + bar_zone / 2, ry + row_h / 2)?;
            }

            render_text(canvas, &tc, &font, "Press any key to exit", Color::RGB(90, 100, 135), cx, py + ph - 18)?;
        }
        Err(_) => {
            // Fallback - no font
            let scale = (stats.max_passed as f32).max(stats.close_calls as f32).max(stats.max_time).max(1.0);
            for (i, (_, value, color)) in rows.iter().enumerate() {
                let num: f32 = value.split_whitespace().next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
                let bw = ((num / scale) * (pw - 40) as f32) as i32;
                let by = py + 60 + i as i32 * 55;
                canvas.set_draw_color(*color);
                canvas.fill_rect(Rect::new(px + 20, by, bw.max(6) as u32, 20))?;
            }
        }
    }
    Ok(())
}

fn render_text(
    canvas: &mut Canvas<Window>,
    tc: &sdl2::render::TextureCreator<sdl2::video::WindowContext>,
    font: &sdl2::ttf::Font,
    text: &str,
    color: Color,
    cx: i32,
    cy: i32,
) -> Result<(), String> {
    let surface = font.render(text).blended(color).map_err(|e| e.to_string())?;
    let texture = tc.create_texture_from_surface(&surface).map_err(|e| e.to_string())?;
    let sdl2::render::TextureQuery { width, height, .. } = texture.query();
    let dst = Rect::new(cx - width as i32 / 2, cy - height as i32 / 2, width, height);
    canvas.copy(&texture, None, dst)
}

/// Try several known system font paths and return the first one that exists.
/// On Ubuntu: install fonts with `sudo apt-get install fonts-dejavu-core`
pub fn find_system_font() -> Option<&'static str> {
    const CANDIDATES: &[&str] = &[
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
        "/usr/share/fonts/dejavu/DejaVuSans.ttf",
        "/System/Library/Fonts/Helvetica.ttc",        // macOS
        "C:/Windows/Fonts/arial.ttf",                 // Windows
        "assets/font.ttf",                            // local override
    ];
    CANDIDATES.iter().copied().find(|p| std::path::Path::new(p).exists())
}