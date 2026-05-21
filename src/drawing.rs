use crate::eadk::{self, Color, Rect, SCREEN_RECT};
use crate::constants::{COLOR_BLACK, COLOR_GREY, LOGO, LOGO_COLORS};

// Beveled square rendering logic matching python's square()
pub fn draw_bevel_square(x: i32, y: i32, c: Color) {
    if x < 0 || x + 10 > 320 || y < 0 || y + 10 > 240 {
        return;
    }
    
    let r8 = (((c.rgb565 >> 11) & 0x1F) * 255 / 31) as i32;
    let g8 = (((c.rgb565 >> 5) & 0x3F) * 255 / 63) as i32;
    let b8 = ((c.rgb565 & 0x1F) * 255 / 31) as i32;
    
    let cd = Color::from_888((r8 / 2) as u16, (g8 / 2) as u16, (b8 / 2) as u16);
    let cl = Color::from_888(((r8 + 255) / 2) as u16, ((g8 + 255) / 2) as u16, ((b8 + 255) / 2) as u16);
    
    eadk::display::push_rect_uniform(Rect { x: (x + 1) as u16, y: (y + 1) as u16, width: 8, height: 8 }, c);
    eadk::display::push_rect_uniform(Rect { x: x as u16, y: y as u16, width: 9, height: 1 }, cl);
    eadk::display::push_rect_uniform(Rect { x: x as u16, y: y as u16, width: 1, height: 9 }, cl);
    eadk::display::push_rect_uniform(Rect { x: x as u16, y: (y + 9) as u16, width: 9, height: 1 }, cd);
    eadk::display::push_rect_uniform(Rect { x: (x + 9) as u16, y: y as u16, width: 1, height: 10 }, cd);
}

// Memory-based grid query
pub fn gpix(x: i32, y: i32, grid: &[[Color; 23]; 12]) -> Color {
    let col = (x - 100) / 10;
    let row = y / 10;
    if col >= 0 && col < 12 && row >= 0 && row < 23 {
        grid[col as usize][row as usize]
    } else {
        COLOR_GREY
    }
}

pub fn draw_logo() {
    let bc = Color::from_888(0, 0, 150);
    let inner_bc = Color::from_888(0, 0, 100);
    
    eadk::display::push_rect_uniform(SCREEN_RECT, COLOR_BLACK);
    eadk::display::push_rect_uniform(Rect { x: 55, y: 20, width: 210, height: 70 }, bc);
    eadk::display::push_rect_uniform(Rect { x: 125, y: 90, width: 70, height: 70 }, bc);
    eadk::display::push_rect_uniform(Rect { x: 60, y: 25, width: 200, height: 60 }, inner_bc);
    eadk::display::push_rect_uniform(Rect { x: 130, y: 75, width: 60, height: 80 }, inner_bc);
    
    for ym in 0..5 {
        for xm in 0..21 {
            let color_idx = LOGO[ym][xm] as usize;
            eadk::display::push_rect_uniform(
                Rect {
                    x: (65 + xm * 9) as u16,
                    y: (32 + ym * 9) as u16,
                    width: 9,
                    height: 9,
                },
                LOGO_COLORS[color_idx],
            );
        }
    }
}
