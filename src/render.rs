// Rendering helpers for drawing the intersection and vehicles.
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

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
