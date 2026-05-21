#![cfg_attr(target_os = "none", no_std)]
#![no_main]

#[allow(unused_imports)]
#[cfg(target_os = "none")]
use cortex_m;

#[cfg(target_os = "none")]
use embedded_alloc::LlffHeap as Heap;

#[global_allocator]
#[cfg(target_os = "none")]
static HEAP: Heap = Heap::empty();

#[cfg(target_os = "none")]
extern crate alloc;

pub mod eadk;

#[used]
#[cfg(target_os = "none")]
#[unsafe(link_section = ".rodata.eadk_app_name")]
pub static EADK_APP_NAME: [u8; 7] = *b"Tetris\0";

#[used]
#[cfg(target_os = "none")]
#[unsafe(link_section = ".rodata.eadk_api_level")]
pub static EADK_APP_API_LEVEL: u32 = 0;

#[used]
#[cfg(target_os = "none")]
#[unsafe(link_section = ".rodata.eadk_app_icon")]
pub static EADK_APP_ICON: [u8; 1520] = *include_bytes!("../target/icon.nwi");


#[cfg(target_os = "none")]
use alloc::format;
#[cfg(not(target_os = "none"))]
use std::format;

use crate::eadk::input::Key;
use crate::eadk::{Color, Point, Rect, SCREEN_RECT};

const COLOR_WHITE: Color = Color::from_888(255, 255, 255);
const COLOR_BLACK: Color = Color::from_888(0, 0, 0);
const COLOR_GREY: Color = Color::from_888(120, 120, 120);

// Custom structure for a Tetramino
struct Tetramino {
    states: &'static [&'static [u8]],
    color_index: usize,
}

// Colors for the Tetraminos
const COLORS: [Color; 7] = [
    Color::from_888(0, 255, 255),   // Cyan (I)
    Color::from_888(255, 255, 0),   // Yellow (O)
    Color::from_888(200, 0, 255),   // Magenta/Purple (T)
    Color::from_888(255, 150, 0),   // Orange (L)
    Color::from_888(0, 0, 255),     // Blue (J)
    Color::from_888(255, 0, 0),     // Z piece
    Color::from_888(0, 255, 0),     // S piece
];

// Definition of the 7 classic Tetraminos and their rotation states
static TETRAMINOS: [Tetramino; 7] = [
    // I piece (t = 0)
    Tetramino {
        states: &[
            &[2, 3, 3, 3, 3],
            &[0, 0, 1, 2, 0, 0, 1, 2, 0, 0, 1, 2, 0, 0, 3],
            &[2, 2, 3, 3, 3, 3],
            &[0, 1, 2, 0, 1, 2, 0, 1, 2, 0, 3],
        ],
        color_index: 0,
    },
    // O piece (t = 1)
    Tetramino {
        states: &[
            &[0, 1, 1, 2, 0, 3, 3],
        ],
        color_index: 1,
    },
    // T piece (t = 2)
    Tetramino {
        states: &[
            &[0, 1, 2, 3, 3, 3],
            &[0, 1, 2, 0, 1, 3, 2, 0, 3],
            &[2, 3, 1, 3, 2, 0, 3],
            &[0, 1, 2, 3, 1, 2, 0, 3],
        ],
        color_index: 2,
    },
    // J piece (t = 3)
    Tetramino {
        states: &[
            &[0, 0, 1, 2, 3, 3, 3],
            &[0, 1, 2, 0, 1, 2, 0, 3, 3],
            &[2, 1, 3, 3, 2, 3],
            &[3, 1, 2, 0, 1, 2, 0, 3],
        ],
        color_index: 3,
    },
    // L piece (t = 4)
    Tetramino {
        states: &[
            &[1, 2, 3, 3, 3],
            &[0, 1, 3, 2, 0, 1, 2, 0, 3],
            &[2, 3, 3, 1, 2, 0, 0, 3],
            &[0, 1, 2, 0, 1, 2, 3, 3],
        ],
        color_index: 4,
    },
    // Z piece (t = 5)
    Tetramino {
        states: &[
            &[3, 1, 2, 0, 3, 3],
            &[0, 0, 1, 2, 0, 1, 3, 2, 0, 3],
            &[2, 3, 1, 2, 0, 3, 3],
            &[0, 1, 2, 1, 3, 2, 3],
        ],
        color_index: 5,
    },
    // S piece (t = 6)
    Tetramino {
        states: &[
            &[0, 1, 3, 2, 3, 3],
            &[0, 1, 2, 0, 3, 1, 2, 0, 0, 3],
            &[2, 0, 1, 3, 2, 3, 3],
            &[1, 2, 3, 1, 2, 0, 3],
        ],
        color_index: 6,
    },
];

// SRS wall kick offsets
const KICKDATA: [[(i32, i32); 4]; 8] = [
    [(-1, 0), (-1, -1), (0, 2), (-1, 2)],
    [(1, 0), (1, 1), (0, -2), (1, -2)],
    [(1, 0), (1, -1), (0, 2), (1, 2)],
    [(-1, 0), (-1, 1), (0, -2), (-1, -2)],
    [(-2, 0), (1, 0), (-2, 1), (1, -2)],
    [(2, 0), (-1, 0), (2, -1), (-1, 2)],
    [(-1, 0), (2, 0), (-1, -2), (2, 1)],
    [(1, 0), (-2, 0), (1, 2), (-2, -1)],
];

const IKICKS: [[usize; 4]; 4] = [
    [0, 5, 0, 7],
    [4, 0, 7, 0],
    [0, 6, 0, 4],
    [6, 0, 5, 0],
];

// Score and levels constants
const SCORESYS: [[i32; 5]; 5] = [
    [0, 100, 300, 500, 800],
    [100, 400, 1200, 1600, 0],
    [0, 1200, 1800, 2400, 1200],
    [100, 200, 400, 1200, 0],
    [0, 0, 600, 2400, 0],
];

const SCORETITLES: [[&str; 5]; 5] = [
    ["         ", "Single", "Double", "Triple", "Tetris"],
    ["M T-S", "T-S S", "T-S D", "T-S T", ""],
    ["", "T-S B", "T-S D B", "T-S T B", "Tetris B"],
    ["M T-S", "M T-S S", "M T-S D", "T-S T", ""],
    ["", "", "T-S M D B", "T-S T B", ""],
];

const BSYS: [[i32; 5]; 5] = [
    [0, 0, 0, 0, 1],
    [1, 1, 1, 1, 0],
    [1, 1, 1, 1, 1],
    [0, 0, 1, 1, 0],
    [0, 0, 1, 1, 0],
];

const LEVELSYS: [i32; 5] = [0, 1, 3, 5, 8];

// Beveled square rendering logic matching python's square()
fn draw_bevel_square(x: i32, y: i32, c: Color) {
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

static mut RNG_STATE: u64 = 123456789;

fn seed_rng(seed: u64) {
    unsafe {
        RNG_STATE = if seed == 0 { 123456789 } else { seed };
    }
}

fn next_random() -> u32 {
    unsafe {
        RNG_STATE = RNG_STATE.wrapping_mul(1664525).wrapping_add(1013904223);
        (RNG_STATE >> 16) as u32
    }
}

// 7-bag randomizer
fn random_bag() -> [usize; 7] {
    let mut bag = [0; 7];
    let mut in_bag = [0, 1, 2, 3, 4, 5, 6];
    let mut len = 7;
    for i in 0..7 {
        let rand_idx = (next_random() as usize) % len;
        bag[i] = in_bag[rand_idx];
        for j in rand_idx..(len - 1) {
            in_bag[j] = in_bag[j + 1];
        }
        len -= 1;
    }
    bag
}

// Memory-based grid query
fn gpix(x: i32, y: i32, grid: &[[Color; 23]; 12]) -> Color {
    let col = (x - 100) / 10;
    let row = y / 10;
    if col >= 0 && col < 12 && row >= 0 && row < 23 {
        grid[col as usize][row as usize]
    } else {
        COLOR_GREY
    }
}

// Write frozen piece to memory grid
fn lock_tetra_in_grid(xt: i32, yt: i32, t: usize, r: usize, grid: &mut [[Color; 23]; 12]) {
    let cur = TETRAMINOS[t].states[r];
    let color = COLORS[TETRAMINOS[t].color_index];
    let mut xl = xt;
    let mut y_curr = yt;
    for &cmd in cur {
        if cmd == 0 {
            xl += 10;
        } else if cmd == 1 || cmd == 3 {
            let col = (xl - 100) / 10;
            let row = y_curr / 10;
            if col >= 0 && col < 12 && row >= 0 && row < 23 {
                grid[col as usize][row as usize] = color;
            }
            draw_bevel_square(xl, y_curr, color);
            xl += 10;
        } else if cmd == 2 {
            y_curr += 10;
            xl = xt;
        }
    }
}

// Draw the tetramino on screen dynamically
fn draw_tetra_on_screen(xt: i32, yt: i32, t: usize, r: usize) {
    let cur = TETRAMINOS[t].states[r];
    let color = COLORS[TETRAMINOS[t].color_index];
    let mut xl = xt;
    let mut y_curr = yt;
    for &cmd in cur {
        if cmd == 0 {
            xl += 10;
        } else if cmd == 1 || cmd == 3 {
            draw_bevel_square(xl, y_curr, color);
            xl += 10;
        } else if cmd == 2 {
            y_curr += 10;
            xl = xt;
        }
    }
}

// Erase the active piece with white
fn refresh_tetra(t: usize, px: i32, py: i32, prot: usize) {
    let cur = TETRAMINOS[t].states[prot];
    let mut xl = px;
    let mut y_curr = py;
    for &cmd in cur {
        if cmd == 0 {
            xl += 10;
        } else if cmd == 1 || cmd == 3 {
            eadk::display::push_rect_uniform(Rect { x: xl as u16, y: y_curr as u16, width: 10, height: 10 }, COLOR_WHITE);
            xl += 10;
        } else if cmd == 2 {
            y_curr += 10;
            xl = px;
        }
    }
}

// Lightened ghost color helper
fn get_ghost_color(color: Color) -> Color {
    let r8 = (((color.rgb565 >> 11) & 0x1F) * 255 / 31) as i32;
    let g8 = (((color.rgb565 >> 5) & 0x3F) * 255 / 63) as i32;
    let b8 = ((color.rgb565 & 0x1F) * 255 / 31) as i32;
    
    let gr = (r8 + 255 * 5) / 6;
    let gg = (g8 + 255 * 5) / 6;
    let gb = (b8 + 255 * 5) / 6;
    
    Color::from_888(gr as u16, gg as u16, gb as u16)
}

fn draw_outline_on_screen(xt: i32, yt: i32, t: usize, r: usize) {
    let cur = TETRAMINOS[t].states[r];
    let color = COLORS[TETRAMINOS[t].color_index];
    let ghost_color = get_ghost_color(color);
    let mut xl = xt;
    let mut y_curr = yt;
    for &cmd in cur {
        if cmd == 0 {
            xl += 10;
        } else if cmd == 1 || cmd == 3 {
            draw_bevel_square(xl, y_curr, ghost_color);
            xl += 10;
        } else if cmd == 2 {
            y_curr += 10;
            xl = xt;
        }
    }
}

// Ghost piece lowest position solver
fn get_lowest(x: i32, y: i32, t: usize, r: usize, grid: &[[Color; 23]; 12]) -> i32 {
    let cur = TETRAMINOS[t].states[r];
    let mut ys = y;
    loop {
        if ys >= 230 {
            return ys - 10;
        }
        let mut py = ys + 5;
        let mut px = x + 5;
        let mut collision = false;
        for &cmd in cur {
            if cmd == 0 {
                px += 10;
            } else if cmd == 1 || cmd == 3 {
                let color = gpix(px, py, grid);
                if color.rgb565 != COLOR_WHITE.rgb565 {
                    collision = true;
                    break;
                }
                px += 10;
            } else if cmd == 2 {
                py += 10;
                px = x + 5;
            }
        }
        if collision {
            return ys - 10;
        }
        ys += 10;
    }
}

// Grid initializer
fn init_grid(grid: &mut [[Color; 23]; 12]) {
    for col in 0..12 {
        for row in 0..23 {
            grid[col][row] = COLOR_WHITE;
        }
    }
    for row in 0..23 {
        grid[0][row] = COLOR_GREY;
        grid[11][row] = COLOR_GREY;
    }
    for col in 0..12 {
        grid[col][22] = COLOR_GREY;
    }
}

fn draw_grid_on_screen(_grid: &[[Color; 23]; 12]) {
    eadk::display::push_rect_uniform(Rect { x: 110, y: 0, width: 100, height: 230 }, COLOR_WHITE);
    for row in 0..23 {
        let y = (row * 10) as i32;
        draw_bevel_square(100, y, COLOR_GREY);
        draw_bevel_square(210, y, COLOR_GREY);
    }
    for col in 1..11 {
        let x = (100 + col * 10) as i32;
        draw_bevel_square(x, 220, COLOR_GREY);
    }
}

// Next Piece Queue management
struct PieceQueue {
    queue: [usize; 14],
    consumed: usize,
}

impl PieceQueue {
    fn new() -> Self {
        let mut pq = PieceQueue {
            queue: [0; 14],
            consumed: 0,
        };
        let bag1 = random_bag();
        let bag2 = random_bag();
        pq.queue[0..7].copy_from_slice(&bag1);
        pq.queue[7..14].copy_from_slice(&bag2);
        pq
    }

    fn pop(&mut self) -> usize {
        let next_piece = self.queue[0];
        for i in 0..13 {
            self.queue[i] = self.queue[i + 1];
        }
        self.consumed += 1;
        if self.consumed >= 7 {
            self.consumed = 0;
            let new_bag = random_bag();
            self.queue[7..14].copy_from_slice(&new_bag);
        }
        next_piece
    }

    fn draw_preview(&self) {
        eadk::display::push_rect_uniform(Rect { x: 250, y: 50, width: 40, height: 130 }, COLOR_BLACK);
        for ns in 0..4 {
            draw_tetra_on_screen(250, 60 + ns as i32 * 30, self.queue[ns], 0);
        }
    }
}

// Speed and gravity calculation
fn get_drop_delay_ms(level: i32) -> u64 {
    let base = 0.8 - (level - 1) as f64 * 0.007;
    let mut result = 1.0;
    for _ in 0..(level - 1) {
        result *= base;
    }
    (result * 1000.0) as u64
}

// Main Logo and background drawing
const LOGO: [[u8; 21]; 5] = [
    [1, 1, 1, 0, 2, 2, 2, 0, 3, 3, 3, 0, 4, 4, 4, 0, 5, 0, 0, 6, 6],
    [0, 1, 0, 0, 2, 0, 0, 0, 0, 3, 0, 0, 4, 0, 4, 0, 0, 0, 6, 0, 0],
    [0, 1, 0, 0, 2, 2, 0, 0, 0, 3, 0, 0, 4, 4, 0, 0, 5, 0, 0, 6, 0],
    [0, 1, 0, 0, 2, 0, 0, 0, 0, 3, 0, 0, 4, 0, 4, 0, 5, 0, 0, 0, 6],
    [0, 1, 0, 0, 2, 2, 2, 0, 0, 3, 0, 0, 4, 0, 4, 0, 5, 0, 6, 6, 0],
];

const LOGO_COLORS: [Color; 7] = [
    Color::from_888(0, 0, 100),
    Color::from_888(255, 0, 0),
    Color::from_888(255, 150, 0),
    Color::from_888(255, 255, 0),
    Color::from_888(0, 255, 0),
    Color::from_888(0, 255, 255),
    Color::from_888(255, 0, 255),
];

fn draw_logo() {
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

fn draw_help_screen() {
    let color_bg_dark = Color::from_888(15, 15, 20);
    let color_panel_bg = Color::from_888(30, 30, 40);
    let color_border = Color::from_888(60, 60, 80);
    let color_accent = Color::from_888(0, 180, 255);
    let color_light_grey = Color::from_888(180, 180, 180);
    
    // Clear screen
    eadk::display::push_rect_uniform(SCREEN_RECT, color_bg_dark);
    
    // Panel card
    let panel_rect = Rect { x: 10, y: 10, width: 300, height: 220 };
    eadk::display::push_rect_uniform(panel_rect, color_panel_bg);
    
    eadk::display::push_rect_uniform(Rect { x: 10, y: 10, width: 2, height: 220 }, color_border);
    eadk::display::push_rect_uniform(Rect { x: 308, y: 10, width: 2, height: 220 }, color_border);
    eadk::display::push_rect_uniform(Rect { x: 10, y: 10, width: 300, height: 2 }, color_border);
    eadk::display::push_rect_uniform(Rect { x: 10, y: 228, width: 300, height: 2 }, color_border);
    
    // Header Bar
    let header_rect = Rect { x: 12, y: 12, width: 296, height: 32 };
    eadk::display::push_rect_uniform(header_rect, Color::from_888(22, 22, 30));
    eadk::display::push_rect_uniform(Rect { x: 12, y: 44, width: 296, height: 2 }, color_accent);
    
    eadk::display::draw_string(
        "REGLES DES MODES",
        Point { x: 80, y: 20 },
        true,
        COLOR_WHITE,
        Color::from_888(22, 22, 30),
    );
    
    // MARATHON
    eadk::display::draw_string(
        "MARATHON (Classique)",
        Point { x: 20, y: 55 },
        false,
        Color::from_888(0, 230, 100), // green
        color_panel_bg,
    );
    eadk::display::draw_string(
        "Faire le meilleur score possible.",
        Point { x: 20, y: 70 },
        false,
        color_light_grey,
        color_panel_bg,
    );
    eadk::display::draw_string(
        "La vitesse augmente toutes les 10 lignes.",
        Point { x: 20, y: 83 },
        false,
        color_light_grey,
        color_panel_bg,
    );
    
    // SPRINT
    eadk::display::draw_string(
        "SPRINT (Contre-la-montre)",
        Point { x: 20, y: 105 },
        false,
        Color::from_888(255, 180, 0), // orange
        color_panel_bg,
    );
    eadk::display::draw_string(
        "Eliminer 40 lignes le plus vite possible.",
        Point { x: 20, y: 120 },
        false,
        color_light_grey,
        color_panel_bg,
    );
    eadk::display::draw_string(
        "Un chrono mesure votre performance.",
        Point { x: 20, y: 133 },
        false,
        color_light_grey,
        color_panel_bg,
    );
    
    // ULTRA
    eadk::display::draw_string(
        "ULTRA (Score-attack)",
        Point { x: 20, y: 155 },
        false,
        color_accent, // cyan
        color_panel_bg,
    );
    eadk::display::draw_string(
        "Obtenir le maximum de points dans un",
        Point { x: 20, y: 170 },
        false,
        color_light_grey,
        color_panel_bg,
    );
    eadk::display::draw_string(
        "temps limite de 3 minutes exactes.",
        Point { x: 20, y: 183 },
        false,
        color_light_grey,
        color_panel_bg,
    );
    
    // Bottom return hint
    eadk::display::push_rect_uniform(Rect { x: 12, y: 205, width: 296, height: 1 }, color_border);
    let r_text = "Pressez OK ou BACK pour quitter";
    let rx = 10 + (300 - (r_text.len() * 8)) / 2;
    eadk::display::draw_string(
        r_text,
        Point { x: rx as u16, y: 212 },
        false,
        COLOR_WHITE,
        color_panel_bg,
    );
    
    // Wait for key release first, then key press
    eadk::timing::msleep(200);
    loop {
        let keys = eadk::input::KeyboardState::scan();
        if keys.key_down(Key::Ok) || keys.key_down(Key::Back) || keys.key_down(Key::Exe) {
            while eadk::input::KeyboardState::scan().key_down(Key::Ok) || eadk::input::KeyboardState::scan().key_down(Key::Back) || eadk::input::KeyboardState::scan().key_down(Key::Exe) {
                eadk::timing::msleep(10);
            }
            break;
        }
        eadk::timing::msleep(16);
    }
}

fn draw_settings_layout() {
    let color_bg_dark = Color::from_888(15, 15, 20);
    let color_panel_bg = Color::from_888(30, 30, 40);
    let color_border = Color::from_888(60, 60, 80);
    let color_accent = Color::from_888(0, 180, 255);      // Cyan neon

    // 1. Draw outer dark background
    eadk::display::push_rect_uniform(SCREEN_RECT, color_bg_dark);
    
    // 2. Draw central panel card with borders
    let panel_rect = Rect { x: 10, y: 10, width: 300, height: 220 };
    eadk::display::push_rect_uniform(panel_rect, color_panel_bg);
    
    eadk::display::push_rect_uniform(Rect { x: 10, y: 10, width: 2, height: 220 }, color_border);
    eadk::display::push_rect_uniform(Rect { x: 308, y: 10, width: 2, height: 220 }, color_border);
    eadk::display::push_rect_uniform(Rect { x: 10, y: 10, width: 300, height: 2 }, color_border);
    eadk::display::push_rect_uniform(Rect { x: 10, y: 228, width: 300, height: 2 }, color_border);

    // 3. Draw Header Bar
    let header_rect = Rect { x: 12, y: 12, width: 296, height: 32 };
    eadk::display::push_rect_uniform(header_rect, Color::from_888(22, 22, 30));
    
    // Cyan divider below header
    eadk::display::push_rect_uniform(Rect { x: 12, y: 44, width: 296, height: 2 }, color_accent);
    
    // Header Title
    eadk::display::draw_string(
        "OPTIONS & CONFIG",
        Point { x: 80, y: 20 },
        true,
        COLOR_WHITE,
        Color::from_888(22, 22, 30),
    );

    // Bottom Help Area Divider
    eadk::display::push_rect_uniform(Rect { x: 12, y: 205, width: 296, height: 1 }, color_border);
}

fn draw_settings_row(row_idx: usize, is_active: bool, mode: usize, speed: i32, auto_level: bool) {
    let modetext = ["Marathon", "Sprint", "Ultra"];
    
    // Premium Color Constants
    let color_panel_bg = Color::from_888(30, 30, 40);
    let color_border = Color::from_888(60, 60, 80);
    let color_accent = Color::from_888(0, 180, 255);      // Cyan neon
    let color_accent_muted = Color::from_888(0, 70, 110);
    let color_light_grey = Color::from_888(180, 180, 180);
    let color_dark_grey = Color::from_888(60, 60, 60);

    match row_idx {
        0 => {
            // Row 0: Mode Selection
            eadk::display::push_rect_uniform(Rect { x: 12, y: 46, width: 296, height: 49 }, color_panel_bg);
            
            let label_color = if is_active { color_accent } else { COLOR_WHITE };
            eadk::display::draw_string(
                "MODE DE JEU",
                Point { x: 25, y: 70 },
                false,
                label_color,
                color_panel_bg,
            );
            
            // Draw Mode Selector Widget Box
            let selector_rect = Rect { x: 140, y: 65, width: 140, height: 20 };
            eadk::display::push_rect_uniform(selector_rect, color_border);
            let selector_inner = Rect { x: 141, y: 66, width: 138, height: 18 };
            eadk::display::push_rect_uniform(selector_inner, Color::from_888(15, 15, 20));
            
            // Center the mode text
            let mode_str = modetext[mode];
            let mode_len = mode_str.len();
            let text_x = 140 + (140 - (mode_len as i32 * 8)) / 2;
            
            let arrow_color = if is_active { color_accent } else { color_light_grey };
            eadk::display::draw_string("<", Point { x: 145, y: 67 }, false, arrow_color, Color::from_888(15, 15, 20));
            eadk::display::draw_string(">", Point { x: 265, y: 67 }, false, arrow_color, Color::from_888(15, 15, 20));
            
            eadk::display::draw_string(
                mode_str,
                Point { x: text_x as u16, y: 67 },
                false,
                COLOR_WHITE,
                Color::from_888(15, 15, 20),
            );

            // Left focus indicator
            if is_active {
                eadk::display::push_rect_uniform(Rect { x: 18, y: 68, width: 3, height: 14 }, color_accent);
            }
        }
        1 => {
            // Row 1: Starting Speed Selection
            eadk::display::push_rect_uniform(Rect { x: 12, y: 95, width: 296, height: 40 }, color_panel_bg);
            
            let label_color = if is_active { color_accent } else { COLOR_WHITE };
            eadk::display::draw_string(
                "VITESSE INIT",
                Point { x: 25, y: 110 },
                false,
                label_color,
                color_panel_bg,
            );
            
            // Draw Speed Segment Slider
            for i in 0..15 {
                let tick_x = 140 + i * 6;
                let tick_rect = Rect { x: tick_x as u16, y: 113, width: 4, height: 12 };
                
                let is_tick_active = i < speed;
                let tick_color = if is_tick_active {
                    if is_active {
                        if i < 5 {
                            Color::from_888(0, 230, 100) // Green
                        } else if i < 10 {
                            Color::from_888(255, 180, 0) // Orange
                        } else {
                            Color::from_888(255, 50, 50)  // Red
                        }
                    } else {
                        color_accent_muted
                    }
                } else {
                    color_dark_grey
                };
                
                eadk::display::push_rect_uniform(tick_rect, tick_color);
            }
            
            // Numeric speed string
            let speed_str = format!("Lv.{:02}", speed);
            eadk::display::draw_string(
                &speed_str,
                Point { x: 235, y: 110 },
                false,
                COLOR_WHITE,
                color_panel_bg,
            );

            // Left focus indicator
            if is_active {
                eadk::display::push_rect_uniform(Rect { x: 18, y: 109, width: 3, height: 14 }, color_accent);
            }
        }
        2 => {
            // Row 2: Auto Level Checkbox
            eadk::display::push_rect_uniform(Rect { x: 12, y: 135, width: 296, height: 35 }, color_panel_bg);
            
            let label_color = if is_active { color_accent } else { COLOR_WHITE };
            eadk::display::draw_string(
                "NIVEAU AUTO",
                Point { x: 25, y: 147 },
                false,
                label_color,
                color_panel_bg,
            );
            
            // Checkbox Box
            let cb_border = Rect { x: 195, y: 145, width: 18, height: 18 };
            let cb_bg = Rect { x: 196, y: 146, width: 16, height: 16 };
            let box_border_color = if is_active { color_accent } else { color_border };
            
            eadk::display::push_rect_uniform(cb_border, box_border_color);
            eadk::display::push_rect_uniform(cb_bg, Color::from_888(15, 15, 20));
            
            if auto_level {
                let cb_check = Rect { x: 199, y: 149, width: 10, height: 10 };
                let check_color = if is_active { color_accent } else { color_light_grey };
                eadk::display::push_rect_uniform(cb_check, check_color);
            }

            // Left focus indicator
            if is_active {
                eadk::display::push_rect_uniform(Rect { x: 18, y: 147, width: 3, height: 14 }, color_accent);
            }
        }
        3 => {
            // Row 3: Save Button
            eadk::display::push_rect_uniform(Rect { x: 12, y: 170, width: 296, height: 35 }, color_panel_bg);
            
            let btn_rect = Rect { x: 50, y: 177, width: 220, height: 22 };
            let btn_border = Rect { x: 48, y: 175, width: 224, height: 26 };
            
            if is_active {
                eadk::display::push_rect_uniform(btn_border, color_accent);
                eadk::display::push_rect_uniform(btn_rect, color_accent);
                eadk::display::draw_string(
                    "SAUVEGARDER & FERMER",
                    Point { x: 68, y: 180 },
                    false,
                    COLOR_BLACK,
                    color_accent,
                );
            } else {
                eadk::display::push_rect_uniform(btn_border, color_border);
                eadk::display::push_rect_uniform(btn_rect, color_panel_bg);
                eadk::display::draw_string(
                    "SAUVEGARDER & FERMER",
                    Point { x: 68, y: 180 },
                    false,
                    color_light_grey,
                    color_panel_bg,
                );
            }

            // Left focus indicator
            if is_active {
                eadk::display::push_rect_uniform(Rect { x: 18, y: 179, width: 3, height: 14 }, color_accent);
            }
        }
        4 => {
            // Row 4: Header Help Button
            let header_bg = Color::from_888(22, 22, 30);
            eadk::display::push_rect_uniform(Rect { x: 260, y: 13, width: 45, height: 30 }, header_bg);
            
            let help_btn_border = Rect { x: 275, y: 17, width: 22, height: 22 };
            let help_btn_bg = Rect { x: 276, y: 18, width: 20, height: 20 };
            
            if is_active {
                eadk::display::push_rect_uniform(help_btn_border, color_accent);
                eadk::display::push_rect_uniform(help_btn_bg, color_accent);
                eadk::display::draw_string(
                    "?",
                    Point { x: 282, y: 21 },
                    false,
                    COLOR_BLACK,
                    color_accent,
                );
                // Glow focus indicator next to it
                eadk::display::push_rect_uniform(Rect { x: 268, y: 21, width: 3, height: 14 }, color_accent);
            } else {
                eadk::display::push_rect_uniform(help_btn_border, color_border);
                eadk::display::push_rect_uniform(help_btn_bg, header_bg);
                eadk::display::draw_string(
                    "?",
                    Point { x: 282, y: 21 },
                    false,
                    color_light_grey,
                    header_bg,
                );
            }
        }
        _ => {}
    }
}

fn draw_settings_help(active_row: usize) {
    let color_panel_bg = Color::from_888(30, 30, 40);
    let color_light_grey = Color::from_888(180, 180, 180);

    // Clear help bar area
    eadk::display::push_rect_uniform(Rect { x: 12, y: 206, width: 296, height: 22 }, color_panel_bg);
    
    let fit_help_text = match active_row {
        0 => "G/D: Changer mode  HAUT/BAS: Bouger",
        1 => "G/D: Vitesse (1-15) HAUT/BAS: Bouger",
        2 => "OK/EXE: Activer/Desactiver niv. auto",
        3 => "OK/EXE: Sauvegarder & Fermer",
        4 => "OK/EXE: Lire les regles des modes",
        _ => "",
    };
    
    let help_len = fit_help_text.len();
    let help_x = 10 + (300 - (help_len * 8)) / 2;
    
    eadk::display::draw_string(
        fit_help_text,
        Point { x: help_x as u16, y: 212 },
        false,
        color_light_grey,
        color_panel_bg,
    );
}

fn draw_settings_panel(active_row: usize, mode: usize, speed: i32, auto_level: bool) {
    for r in 0..4 {
        draw_settings_row(r, r == active_row, mode, speed, auto_level);
    }
    draw_settings_row(4, active_row == 4, mode, speed, auto_level);
    draw_settings_help(active_row);
}

fn draw_main_menu_extras() {
    eadk::display::draw_string("Press [EXE] to start", Point { x: 60, y: 170 }, false, COLOR_WHITE, COLOR_BLACK);
    eadk::display::draw_string("[0] Options", Point { x: 105, y: 190 }, false, COLOR_WHITE, COLOR_BLACK);
    
    // Premium Help capsule button in the top-right corner
    let btn_border = Rect { x: 215, y: 10, width: 95, height: 22 };
    let btn_bg = Rect { x: 216, y: 11, width: 93, height: 20 };
    let border_color = Color::from_888(60, 60, 80);
    let bg_color = Color::from_888(25, 25, 35);
    let accent_color = Color::from_888(0, 180, 255);
    
    eadk::display::push_rect_uniform(btn_border, border_color);
    eadk::display::push_rect_uniform(btn_bg, bg_color);
    eadk::display::draw_string("[Toolbox]", Point { x: 221, y: 14 }, false, COLOR_WHITE, bg_color);
    eadk::display::draw_string("?", Point { x: 295, y: 14 }, false, accent_color, bg_color);
}

// Menu screen with options
fn show_menu() -> (usize, i32, bool) {
    let mut mode = 0;
    let mut speed = 2;
    let mut auto_level = true;
    let mut menu_page = 0;
    let mut active_row = 0;
    
    draw_logo();
    draw_main_menu_extras();
    
    loop {
        let keys = eadk::input::KeyboardState::scan();
        
        if menu_page == 0 {
            if keys.key_down(Key::Toolbox) {
                draw_help_screen();
                // After returning from help, clean and redraw main menu
                eadk::display::push_rect_uniform(SCREEN_RECT, COLOR_BLACK);
                draw_logo();
                draw_main_menu_extras();
                while eadk::input::KeyboardState::scan().key_down(Key::Toolbox) {
                    eadk::timing::msleep(10);
                }
            }
            if keys.key_down(Key::Zero) {
                menu_page = 1;
                active_row = 0;
                
                draw_settings_layout();
                // Initial draw of the settings panel
                draw_settings_panel(active_row, mode, speed, auto_level);
                
                while eadk::input::KeyboardState::scan().key_down(Key::Zero) {
                    eadk::timing::msleep(10);
                }
            }
            if keys.key_down(Key::Exe) {
                while eadk::input::KeyboardState::scan().key_down(Key::Exe) {
                    eadk::timing::msleep(10);
                }
                break;
            }
        } else if menu_page == 1 {
            let mut state_changed = false;
            
            if keys.key_down(Key::Up) {
                let old_row = active_row;
                if active_row == 0 {
                    active_row = 4;
                    state_changed = true;
                } else if active_row == 4 {
                    // Do nothing or wrap around
                } else {
                    active_row -= 1;
                    state_changed = true;
                }
                if state_changed {
                    draw_settings_row(old_row, false, mode, speed, auto_level);
                    draw_settings_row(active_row, true, mode, speed, auto_level);
                    draw_settings_help(active_row);
                    state_changed = false; // Bypassed full redraw!
                }
                eadk::timing::msleep(150);
            } else if keys.key_down(Key::Down) {
                let old_row = active_row;
                if active_row == 4 {
                    active_row = 0;
                    state_changed = true;
                } else if active_row < 3 {
                    active_row += 1;
                    state_changed = true;
                }
                if state_changed {
                    draw_settings_row(old_row, false, mode, speed, auto_level);
                    draw_settings_row(active_row, true, mode, speed, auto_level);
                    draw_settings_help(active_row);
                    state_changed = false; // Bypassed full redraw!
                }
                eadk::timing::msleep(150);
            }
            
            if active_row == 0 {
                if keys.key_down(Key::Left) {
                    mode = if mode == 0 { 2 } else { mode - 1 };
                    draw_settings_row(0, true, mode, speed, auto_level);
                    eadk::timing::msleep(150);
                } else if keys.key_down(Key::Right) {
                    mode = (mode + 1) % 3;
                    draw_settings_row(0, true, mode, speed, auto_level);
                    eadk::timing::msleep(150);
                }
            } else if active_row == 1 {
                if keys.key_down(Key::Left) {
                    if speed > 1 {
                        speed -= 1;
                        draw_settings_row(1, true, mode, speed, auto_level);
                    }
                    eadk::timing::msleep(100);
                } else if keys.key_down(Key::Right) {
                    if speed < 15 {
                        speed += 1;
                        draw_settings_row(1, true, mode, speed, auto_level);
                    }
                    eadk::timing::msleep(100);
                }
            } else if active_row == 2 {
                if keys.key_down(Key::Ok) || keys.key_down(Key::Exe) {
                    auto_level = !auto_level;
                    draw_settings_row(2, true, mode, speed, auto_level);
                    while eadk::input::KeyboardState::scan().key_down(Key::Ok) || eadk::input::KeyboardState::scan().key_down(Key::Exe) {
                        eadk::timing::msleep(10);
                    }
                }
            } else if active_row == 4 {
                if keys.key_down(Key::Ok) || keys.key_down(Key::Exe) {
                    draw_help_screen();
                    // After returning, redraw options panel
                    draw_settings_layout();
                    draw_settings_panel(active_row, mode, speed, auto_level);
                }
            } else if active_row == 3 {
                if keys.key_down(Key::Ok) || keys.key_down(Key::Exe) {
                    menu_page = 0;
                    eadk::display::push_rect_uniform(SCREEN_RECT, COLOR_BLACK);
                    draw_logo();
                    draw_main_menu_extras();
                    while eadk::input::KeyboardState::scan().key_down(Key::Ok) || eadk::input::KeyboardState::scan().key_down(Key::Exe) {
                        eadk::timing::msleep(10);
                    }
                }
            }
            
            if keys.key_down(Key::Back) || keys.key_down(Key::Zero) {
                menu_page = 0;
                eadk::display::push_rect_uniform(SCREEN_RECT, COLOR_BLACK);
                draw_logo();
                draw_main_menu_extras();
                while eadk::input::KeyboardState::scan().key_down(Key::Back) || eadk::input::KeyboardState::scan().key_down(Key::Zero) {
                    eadk::timing::msleep(10);
                }
            }
            
            if state_changed {
                draw_settings_panel(active_row, mode, speed, auto_level);
            }
        }
        eadk::timing::msleep(16);
    }
    
    (mode, speed, auto_level)
}

#[derive(PartialEq)]
enum GameOutcome {
    Menu,
    Restart,
}

// Main Game loop logic
fn run_game(mode: usize, speed: i32, auto_level: bool) -> GameOutcome {
    let mut x = 140;
    let mut y = 20;
    let mut px = 150;
    let mut py = 10;
    let mut t = 0;
    let mut wait_end_time: Option<u64> = None;
    let mut rot = 0;
    let mut prot = 0;
    let mut plowest = 500;
    
    let mut piece_queue = PieceQueue::new();
    t = piece_queue.pop();
    
    let mut hold = -1;
    let mut hold_allowed = true;
    let mut score = 0;
    let mut combo = 0;
    let mut level = speed;
    let mut xp = 0;
    let mut btob = 0;
    let mut lines = 0;
    let mut end = false;
    
    let start_time = eadk::timing::millis();
    let mut last_drop_time = eadk::timing::millis();
    let mut coolh_time = eadk::timing::millis();
    let mut arr = true;
    let mut coolr = 0;
    let mut last_move = -1;
    
    let mut grid = [[COLOR_WHITE; 23]; 12];
    init_grid(&mut grid);

    // Clear the entire screen to black to wipe out menu elements and the logo
    eadk::display::push_rect_uniform(SCREEN_RECT, COLOR_BLACK);

    draw_grid_on_screen(&grid);
    
    // Draw panels and previews border
    eadk::display::push_rect_uniform(Rect { x: 110, y: 19, width: 100, height: 1 }, Color::from_888(255, 0, 0));
    
    for ys in (40..190).step_by(10) {
        for xs in (240..300).step_by(10) {
            draw_bevel_square(xs, ys, COLOR_GREY);
        }
    }
    for ys in (80..140).step_by(10) {
        for xs in (240..300).step_by(10) {
            draw_bevel_square(xs - 220, ys - 40, COLOR_GREY);
        }
    }
    eadk::display::push_rect_uniform(Rect { x: 250, y: 50, width: 40, height: 130 }, COLOR_BLACK);
    eadk::display::push_rect_uniform(Rect { x: 30, y: 50, width: 40, height: 40 }, COLOR_BLACK);
    
    piece_queue.draw_preview();
    
    eadk::display::draw_string(
        if mode != 1 { "Score :" } else { "Lines :" },
        Point { x: 15, y: 110 },
        false,
        COLOR_WHITE,
        COLOR_BLACK,
    );
    eadk::display::draw_string(
        if mode != 1 { "0000000" } else { "00/40" },
        Point { x: 15, y: 130 },
        false,
        COLOR_WHITE,
        COLOR_BLACK,
    );
    eadk::display::draw_string(
        if mode == 0 { "Level :" } else { "Time :" },
        Point { x: 15, y: 160 },
        false,
        COLOR_WHITE,
        COLOR_BLACK,
    );
    if mode == 0 {
        let level_str = format!("{:02}", level);
        eadk::display::draw_string(&level_str, Point { x: 65, y: 180 }, false, COLOR_WHITE, COLOR_BLACK);
    } else {
        eadk::display::draw_string("00:00.00", Point { x: 15, y: 180 }, false, COLOR_WHITE, COLOR_BLACK);
    }
    
    // Countdown
    for tm in 0..3 {
        let tm_str = match tm {
            0 => "3",
            1 => "2",
            _ => "1",
        };
        eadk::display::draw_string(
            tm_str,
            Point { x: 155, y: 110 },
            false,
            COLOR_BLACK,
            COLOR_WHITE,
        );
        eadk::timing::msleep(1000);
    }
    eadk::display::draw_string(
        " ",
        Point { x: 155, y: 110 },
        false,
        COLOR_BLACK,
        COLOR_WHITE,
    );
    
    while !end {
        px = x;
        py = y;
        prot = rot;
        let mut move_arr = [0; 4];
        let keys = eadk::input::KeyboardState::scan();
        let now = eadk::timing::millis();
        
        // Pause screen
        if keys.key_down(Key::Exe) {
            while eadk::input::KeyboardState::scan().key_down(Key::Exe) {
                eadk::timing::msleep(10);
            }
            
            let mut choice = GameOutcome::Menu;
            loop {
                eadk::display::draw_string("Paused", Point { x: 240, y: 205 }, false, COLOR_WHITE, COLOR_BLACK);
                eadk::display::draw_string("7-Restart", Point { x: 5, y: 5 }, false, COLOR_WHITE, COLOR_BLACK);
                eadk::display::draw_string("Menu-9", Point { x: 240, y: 5 }, false, COLOR_WHITE, COLOR_BLACK);
                
                let p_keys = eadk::input::KeyboardState::scan();
                if p_keys.key_down(Key::Seven) {
                    while eadk::input::KeyboardState::scan().key_down(Key::Seven) {
                        eadk::timing::msleep(10);
                    }
                    choice = GameOutcome::Restart;
                    break;
                }
                if p_keys.key_down(Key::Nine) {
                    while eadk::input::KeyboardState::scan().key_down(Key::Nine) {
                        eadk::timing::msleep(10);
                    }
                    choice = GameOutcome::Menu;
                    break;
                }
                if p_keys.key_down(Key::Exe) {
                    while eadk::input::KeyboardState::scan().key_down(Key::Exe) {
                        eadk::timing::msleep(10);
                    }
                    choice = GameOutcome::Restart; // dummy value to resume
                    break;
                }
                eadk::timing::msleep(16);
            }
            
            eadk::display::draw_string("      ", Point { x: 240, y: 205 }, false, COLOR_WHITE, COLOR_BLACK);
            eadk::display::draw_string("         ", Point { x: 5, y: 5 }, false, COLOR_WHITE, COLOR_BLACK);
            eadk::display::draw_string("      ", Point { x: 240, y: 5 }, false, COLOR_WHITE, COLOR_BLACK);
            
            if choice == GameOutcome::Menu {
                return GameOutcome::Menu;
            } else if choice == GameOutcome::Restart && eadk::input::KeyboardState::scan().key_down(Key::Seven) {
                return GameOutcome::Restart;
            }
            
            // Resume countdown
            for tm in 0..3 {
                let tm_str = match tm {
                    0 => "3",
                    1 => "2",
                    _ => "1",
                };
                eadk::display::draw_string(tm_str, Point { x: 10, y: 200 }, false, COLOR_WHITE, COLOR_BLACK);
                eadk::timing::msleep(1000);
            }
            eadk::display::draw_string(" ", Point { x: 10, y: 200 }, false, COLOR_WHITE, COLOR_BLACK);
            last_drop_time = eadk::timing::millis();
            coolh_time = eadk::timing::millis();
        }
        
        // Gravity & Soft drop
        let drop_delay = get_drop_delay_ms(level);
        let is_soft = keys.key_down(Key::Down);
        let drop_check = if is_soft {
            now >= last_drop_time + 30
        } else {
            now >= last_drop_time + drop_delay
        };
        
        if drop_check && !keys.key_down(Key::Up) {
            y += 10;
            last_drop_time = now;
            move_arr[1] = 1;
        }
        
        // Hold
        if keys.key_down(Key::Backspace) && hold_allowed {
            if hold == -1 {
                hold = piece_queue.pop() as i32;
                piece_queue.draw_preview();
            }
            refresh_tetra(t, px, py, prot);
            refresh_tetra(t, px, plowest, prot);
            
            let hold_temp = hold;
            hold = t as i32;
            
            eadk::display::push_rect_uniform(Rect { x: 30, y: 50, width: 40, height: 40 }, COLOR_BLACK);
            draw_tetra_on_screen(30, 60, hold as usize, 0);
            
            t = hold_temp as usize;
            hold_allowed = false;
            
            x = 140;
            y = 10;
            rot = 0;
            px = 140;
            py = 10;
            prot = 0;
            plowest = 500;
        }
        
        // Horizontal moves (DAS)
        if keys.key_down(Key::Left) && now >= coolh_time {
            x -= 10;
            let shift_delay = 30 + if arr { 140 } else { 0 };
            coolh_time = now + shift_delay;
            arr = false;
            move_arr[0] = 1;
        }
        if !keys.key_down(Key::Left) && !keys.key_down(Key::Right) {
            coolh_time = now;
            arr = true;
        }
        if keys.key_down(Key::Right) && now >= coolh_time {
            x += 10;
            let shift_delay = 30 + if arr { 140 } else { 0 };
            coolh_time = now + shift_delay;
            arr = false;
            move_arr[0] = 1;
        }
        
        // Hard Drop
        if keys.key_down(Key::Up) {
            move_arr[3] = 1;
            y = plowest;
            wait_end_time = Some(eadk::timing::millis()); // lock immediately
        }
        
        // Rotation (Ok)
        if keys.key_down(Key::Ok) {
            coolr += 1;
        } else {
            coolr = 0;
        }
        if coolr == 1 {
            let rot_dir = if keys.key_down(Key::Shift) { -1 } else { 1 };
            let count = TETRAMINOS[t].states.len() as i32;
            rot = (rot as i32 + rot_dir).rem_euclid(count) as usize;
            move_arr[2] = 1;
        }
        
        // Update Sprint & Ultra Timer
        let elapsed_ms = eadk::timing::millis() - start_time;
        let timem = (elapsed_ms / 60000) as i32;
        let times_val = ((elapsed_ms % 60000) / 1000) as i32;
        let times = format!("{:02}", times_val);
        let timems_val = ((elapsed_ms % 1000) / 10) as i32;
        let timems = format!("{:02}", timems_val);
        
        if mode != 0 {
            eadk::display::draw_string(&timems, Point { x: 75, y: 180 }, false, COLOR_WHITE, COLOR_BLACK);
            let timem_str = format!("{}", timem);
            eadk::display::draw_string(&timem_str, Point { x: (25 - timem_str.len() as i32 * 10) as u16, y: 180 }, false, COLOR_WHITE, COLOR_BLACK);
            eadk::display::draw_string(&times, Point { x: (65 - times.len() as i32 * 10) as u16, y: 180 }, false, COLOR_WHITE, COLOR_BLACK);
        }
        
        // Check Victory conditions
        if (mode == 1 && lines >= 40) || (mode == 2 && timem >= 3) {
            while eadk::input::KeyboardState::scan().key_down(Key::Ok) {
                eadk::timing::msleep(10);
            }
            eadk::display::draw_string("Victory", Point { x: 125, y: 60 }, false, COLOR_BLACK, COLOR_WHITE);
            eadk::display::draw_string("[OK] Back to menu", Point { x: 75, y: 80 }, false, COLOR_BLACK, COLOR_WHITE);
            loop {
                if eadk::input::KeyboardState::scan().key_down(Key::Ok) {
                    while eadk::input::KeyboardState::scan().key_down(Key::Ok) {
                        eadk::timing::msleep(10);
                    }
                    break;
                }
                eadk::timing::msleep(16);
            }
            return GameOutcome::Menu;
        }
        
        if px != x || py != y || prot != rot {
            refresh_tetra(t, px, py, prot);
            refresh_tetra(t, px, plowest, prot);
            
            eadk::display::push_rect_uniform(Rect { x: 110, y: 19, width: 100, height: 1 }, Color::from_888(255, 0, 0));
            
            let mut x_curr = x;
            let mut y_curr = y;
            let mut is_kicked = false;
            let num_tests = if prot == rot { 3 } else { 7 };
            
            for tests in 0..num_tests {
                let cur = TETRAMINOS[t].states[rot];
                let mut tempx = x_curr;
                let mut tempy = y_curr;
                let mut collision = false;
                
                for &cmd in cur {
                    if cmd == 0 {
                        tempx += 10;
                    } else if cmd == 1 || cmd == 3 {
                        let color = gpix(tempx, tempy, &grid);
                        if color.rgb565 != COLOR_WHITE.rgb565 {
                            collision = true;
                            if tests == 0 {
                                y = py;
                                y_curr = py;
                                move_arr[1] = 0;
                            } else if tests == 1 {
                                x = px;
                                x_curr = px;
                                move_arr[0] = 0;
                            } else if tests >= 2 && tests < 6 && prot != rot {
                                is_kicked = tests < 5;
                                x_curr = x;
                                y_curr = y;
                                if t == 0 {
                                    let ikick = IKICKS[rot][prot];
                                    x_curr += KICKDATA[ikick][tests - 2].0 * 10;
                                    y_curr += KICKDATA[ikick][tests - 2].1 * 10;
                                } else if t >= 2 {
                                    if prot == 0 || prot == 2 {
                                        x_curr += KICKDATA[rot - 1][tests - 2].0 * 10;
                                        y_curr += KICKDATA[rot - 1][tests - 2].1 * 10;
                                    } else {
                                        x_curr += KICKDATA[prot][tests - 2].0 * 10;
                                        y_curr += KICKDATA[prot][tests - 2].1 * 10;
                                    }
                                } else {
                                    rot = prot;
                                    move_arr[2] = 0;
                                    break;
                                }
                            } else if tests == 6 {
                                rot = prot;
                                move_arr[2] = 0;
                            }
                            break;
                        }
                        tempx += 10;
                    } else if cmd == 2 {
                        tempy += 10;
                        tempx = x_curr;
                    }
                }
                if !collision {
                    break;
                }
            }
            
            if prot != rot {
                x = x_curr;
                y = y_curr;
            }
            
            let mut has_moved = true;
            if move_arr[3] == 1 {
                last_move = 3;
                has_moved = false;
            } else if move_arr[2] == 1 {
                last_move = 2;
            } else if move_arr[1] == 1 {
                last_move = 1;
            } else if move_arr[0] == 1 {
                last_move = 0;
            } else {
                has_moved = false;
            }
            
            let cur = TETRAMINOS[t].states[rot];
            let mut tempx = x + 2;
            let mut tempy = y + 2;
            for &cmd in cur {
                if cmd <= 1 {
                    tempx += 10;
                }
                if cmd == 3 {
                    let next_color = gpix(tempx, tempy + 10, &grid);
                    if next_color.rgb565 != COLOR_WHITE.rgb565 {
                        if wait_end_time.is_none() || has_moved {
                            wait_end_time = Some(eadk::timing::millis() + 500);
                        }
                        py = y;
                        if let Some(w_time) = wait_end_time {
                            if eadk::timing::millis() >= w_time {
                                wait_end_time = None;
                                lock_tetra_in_grid(x, y, t, rot, &mut grid);
                                hold_allowed = true;
                                
                                let mut sline = 0;
                                for ys_ in 0..20 {
                                    let ys = 20 - ys_;
                                    let mut line = 0;
                                    for xs in (112..212).step_by(10) {
                                        if gpix(xs, 12 + ys * 10, &grid).rgb565 != COLOR_WHITE.rgb565 {
                                            line += 1;
                                        }
                                    }
                                    if line == 10 {
                                        sline += 1;
                                    }
                                }
                                
                                let mut scoring = 0;
                                if sline == 0 {
                                    combo = 0;
                                } else {
                                    score += combo * 50 * level;
                                    combo += 1;
                                }
                                
                                if t == 2 && last_move == 2 {
                                    let mut c = 0;
                                    for adj in 0..4 {
                                        let corner_x = 5 + x + (adj % 2) * 20;
                                        let corner_y = 5 + y + (adj / 2) * 20;
                                        let ads = gpix(corner_x, corner_y, &grid);
                                        if ads.rgb565 != COLOR_WHITE.rgb565 {
                                            c += 1;
                                        } else {
                                            if (rot % 2 == 0 && (adj as usize == rot || adj as usize == rot + 1))
                                                || (rot % 2 == 1 && (adj as usize == (1 - (rot as i32 - 1) / 2) as usize || adj as usize == (3 - (rot as i32 - 1) / 2) as usize))
                                            {
                                                scoring = 3;
                                            }
                                        }
                                    }
                                    if c >= 3 {
                                        if scoring != 3 {
                                            scoring = 1;
                                        }
                                        if is_kicked {
                                            scoring = 3;
                                        }
                                    } else if scoring == 3 {
                                        scoring = 0;
                                    }
                                }
                                
                                xp += LEVELSYS[sline as usize];
                                if auto_level && xp >= 10 && mode % 3 == 0 {
                                    if level < 15 {
                                        level += 1;
                                    }
                                    xp = 0;
                                }
                                
                                if BSYS[scoring][sline as usize] == 1 {
                                    btob += 1;
                                } else if sline != 0 {
                                    btob = 0;
                                }
                                
                                if btob >= 2 {
                                    if scoring != 3 {
                                        scoring = 2;
                                    } else {
                                        scoring = 4;
                                    }
                                }
                                
                                lines += sline;
                                if mode == 0 {
                                    score += SCORESYS[scoring][sline as usize] * level;
                                } else {
                                    score += SCORESYS[scoring][sline as usize];
                                }
                                
                                let title = SCORETITLES[scoring][sline as usize];
                                
                                if mode != 1 {
                                    let score_str = format!("{}", score);
                                    let score_x = 85 - (score_str.len() as i32) * 10;
                                    eadk::display::draw_string(&score_str, Point { x: score_x as u16, y: 130 }, false, COLOR_WHITE, COLOR_BLACK);
                                } else {
                                    let lines_str = format!("{}", lines);
                                    let lines_x = 35 - (lines_str.len() as i32) * 10;
                                    eadk::display::draw_string(&lines_str, Point { x: lines_x as u16, y: 130 }, false, COLOR_WHITE, COLOR_BLACK);
                                }
                                
                                if mode == 0 {
                                    let level_str = format!("{}", level);
                                    let level_x = 85 - (level_str.len() as i32) * 10;
                                    eadk::display::draw_string(&level_str, Point { x: level_x as u16, y: 180 }, false, COLOR_WHITE, COLOR_BLACK);
                                }
                                
                                eadk::display::draw_string(SCORETITLES[0][0], Point { x: 5, y: 5 }, false, COLOR_WHITE, COLOR_BLACK);
                                eadk::display::draw_string(title, Point { x: 5, y: 5 }, false, COLOR_WHITE, COLOR_BLACK);
                                
                                if BSYS[scoring][sline as usize] == 1 {
                                    btob = 1;
                                }
                                
                                if sline != 0 || scoring != 0 {
                                    eadk::timing::msleep(500);
                                }
                                
                                // Beautiful custom drop gravity matching python lines 305-333
                                for ys_ in 0..20 {
                                    let ys = 20 - ys_;
                                    let mut line = 0;
                                    for xs in (112..212).step_by(10) {
                                        if gpix(xs, 12 + ys * 10, &grid).rgb565 != COLOR_WHITE.rgb565 {
                                            line += 1;
                                        }
                                    }
                                    
                                    if line == 10 {
                                        for xs in 0..10 {
                                            grid[1 + xs][(1 + ys) as usize] = COLOR_WHITE;
                                        }
                                        eadk::display::push_rect_uniform(Rect { x: 110, y: (10 + ys * 10) as u16, width: 100, height: 10 }, COLOR_WHITE);
                                    }
                                    
                                    if line != 0 {
                                        let mut temp_y = 10 + ys * 10;
                                        let mut temp_line = [COLOR_WHITE; 10];
                                        for xs in 0..10 {
                                            temp_line[xs] = grid[1 + xs][(temp_y / 10) as usize];
                                        }
                                        
                                        for xs in 0..10 {
                                            grid[1 + xs][(temp_y / 10) as usize] = COLOR_WHITE;
                                        }
                                        
                                        let mut a = 0;
                                        while a < 10 {
                                            a = 0;
                                            let next_row = (temp_y + 10) / 10;
                                            for xs in 0..10 {
                                                if grid[1 + xs][next_row as usize].rgb565 != COLOR_WHITE.rgb565 {
                                                    a += 1;
                                                }
                                            }
                                            if a == 0 {
                                                eadk::display::push_rect_uniform(Rect { x: 110, y: temp_y as u16, width: 100, height: 10 }, COLOR_WHITE);
                                                temp_y += 10;
                                            } else {
                                                let dest_row = temp_y / 10;
                                                for xs in 0..10 {
                                                    grid[1 + xs][dest_row as usize] = temp_line[xs];
                                                    if temp_line[xs].rgb565 != COLOR_WHITE.rgb565 {
                                                        draw_bevel_square(110 + xs as i32 * 10, temp_y, temp_line[xs]);
                                                    }
                                                }
                                                a = 10;
                                            }
                                        }
                                    }
                                }
                                
                                for xd in (112..212).step_by(10) {
                                    if gpix(xd, 12, &grid).rgb565 != COLOR_WHITE.rgb565 {
                                        end = true;
                                    }
                                }
                                
                                x = 140;
                                y = 0;
                                rot = 0;
                                is_kicked = false;
                                px = x;
                                py = y;
                                prot = rot;
                                
                                t = piece_queue.pop();
                                piece_queue.draw_preview();
                                eadk::timing::msleep(200);
                                break;
                            }
                        }
                    }
                    tempx += 10;
                } else if cmd == 2 {
                    tempy += 10;
                    tempx = x;
                }
            }
            
            plowest = get_lowest(x, y, t, rot, &grid);
            draw_outline_on_screen(x, plowest, t, rot);
        }
        
        draw_tetra_on_screen(x, y, t, rot);
        eadk::timing::msleep(16);
    }
    
    // Game Over
    eadk::display::draw_string("GAME OVER", Point { x: 115, y: 60 }, false, COLOR_BLACK, COLOR_WHITE);
    eadk::display::draw_string("[OK] Back to menu", Point { x: 75, y: 80 }, false, COLOR_BLACK, COLOR_WHITE);
    loop {
        if eadk::input::KeyboardState::scan().key_down(Key::Ok) {
            while eadk::input::KeyboardState::scan().key_down(Key::Ok) {
                eadk::timing::msleep(10);
            }
            break;
        }
        eadk::timing::msleep(16);
    }
    
    GameOutcome::Menu
}

#[unsafe(no_mangle)]
fn main() -> isize {
    #[cfg(target_os = "none")]
    {
        let heap_size: usize = eadk::heap_size();
        unsafe { HEAP.init(eadk::HEAP_START as usize, heap_size) }
    }

    seed_rng(eadk::timing::millis());

    while eadk::input::KeyboardState::scan().key_down(eadk::input::Key::Ok) {
        eadk::timing::msleep(50);
    }

    loop {
        let (mode, speed, auto_level) = show_menu();
        loop {
            let outcome = run_game(mode, speed, auto_level);
            if outcome == GameOutcome::Menu {
                break;
            }
        }
    }
}
