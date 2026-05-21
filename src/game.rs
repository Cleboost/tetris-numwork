#[cfg(target_os = "none")]
use alloc::format;
#[cfg(not(target_os = "none"))]
use std::format;

use crate::eadk::{self, Color, Point, Rect, SCREEN_RECT};
use crate::eadk::input::Key;
use crate::constants::{
    TETRAMINOS, COLORS, COLOR_WHITE, COLOR_BLACK, COLOR_GREY, KICKDATA, IKICKS,
    SCORESYS, SCORETITLES, BSYS, LEVELSYS, GameOutcome
};
use crate::drawing::{draw_bevel_square, gpix};
use crate::piece::PieceQueue;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SavedGameState {
    pub magic: [u8; 4],
    pub mode: u32,
    pub speed: i32,
    pub auto_level: u8,
    pub x: i32,
    pub y: i32,
    pub rot: u32,
    pub t: u32,
    pub hold: i32,
    pub hold_allowed: u8,
    pub score: i32,
    pub combo: i32,
    pub level: i32,
    pub xp: i32,
    pub btob: i32,
    pub lines: i32,
    pub elapsed_ms: u64,
    pub queue: [u32; 14],
    pub consumed: u32,
    pub grid: [u16; 276],
}

impl SavedGameState {
    pub fn as_bytes(&self) -> &[u8] {
        let ptr = self as *const Self as *const u8;
        let len = core::mem::size_of::<Self>();
        unsafe { core::slice::from_raw_parts(ptr, len) }
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != core::mem::size_of::<Self>() {
            return None;
        }
        let mut state = core::mem::MaybeUninit::<Self>::uninit();
        unsafe {
            core::ptr::copy_nonoverlapping(
                bytes.as_ptr(),
                state.as_mut_ptr() as *mut u8,
                core::mem::size_of::<Self>(),
            );
            let state = state.assume_init();
            if state.magic == *b"TETR" {
                Some(state)
            } else {
                None
            }
        }
    }
}

pub const SAVE_FILENAME: &str = "tetris_save.dat";

pub fn save_game_state_to_file(state: &SavedGameState) -> bool {
    crate::storage_lib::storage_file_write(SAVE_FILENAME, state.as_bytes())
}

pub fn load_game_state_from_file() -> Option<SavedGameState> {
    if let Some(data) = crate::storage_lib::storage_extapp_file_read(SAVE_FILENAME) {
        SavedGameState::from_bytes(&data)
    } else {
        None
    }
}

pub fn delete_game_save_file() {
    // Always try to erase, don't rely on exists() check
    crate::storage_lib::storage_extapp_file_erase(SAVE_FILENAME);
}

pub fn auto_save_checkpoint(
    mode: usize,
    speed: i32,
    auto_level: bool,
    x: i32,
    y: i32,
    rot: u32,
    t: u32,
    hold: i32,
    hold_allowed: bool,
    score: i32,
    combo: i32,
    level: i32,
    xp: i32,
    btob: i32,
    lines: i32,
    elapsed_ms: u64,
    piece_queue: &PieceQueue,
    grid: &[[Color; 23]; 12],
) {
    let mut saved_grid = [0u16; 276];
    for col in 0..12 {
        for row in 0..23 {
            saved_grid[col * 23 + row] = grid[col][row].rgb565;
        }
    }

    let mut q = [0u32; 14];
    for i in 0..14 {
        q[i] = piece_queue.queue[i] as u32;
    }

    let state = SavedGameState {
        magic: *b"TETR",
        mode: mode as u32,
        speed,
        auto_level: if auto_level { 1 } else { 0 },
        x,
        y,
        rot,
        t,
        hold,
        hold_allowed: if hold_allowed { 1 } else { 0 },
        score,
        combo,
        level,
        xp,
        btob,
        lines,
        elapsed_ms,
        queue: q,
        consumed: piece_queue.consumed as u32,
        grid: saved_grid,
    };

    save_game_state_to_file(&state);
}

// Grid initializer
pub fn init_grid(grid: &mut [[Color; 23]; 12]) {
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

pub fn draw_grid_on_screen(grid: &[[Color; 23]; 12]) {
    eadk::display::push_rect_uniform(Rect { x: 110, y: 0, width: 100, height: 220 }, COLOR_WHITE);
    for row in 0..23 {
        let y = (row * 10) as i32;
        draw_bevel_square(100, y, COLOR_GREY);
        draw_bevel_square(210, y, COLOR_GREY);
    }
    for col in 1..11 {
        let x = (100 + col * 10) as i32;
        draw_bevel_square(x, 220, COLOR_GREY);
    }
    
    // Render restored blocks
    for col in 1..11 {
        for row in 0..22 {
            let color = grid[col][row];
            if color.rgb565 != COLOR_WHITE.rgb565 {
                let x = 100 + col as i32 * 10;
                let y = row as i32 * 10;
                draw_bevel_square(x, y, color);
            }
        }
    }
}

// Speed and gravity calculation
pub fn get_drop_delay_ms(level: i32) -> u64 {
    let base = 0.8 - (level - 1) as f64 * 0.007;
    let mut result = 1.0;
    for _ in 0..(level - 1) {
        result *= base;
    }
    (result * 1000.0) as u64
}

// Lightened ghost color helper
pub fn get_ghost_color(color: Color) -> Color {
    let r8 = (((color.rgb565 >> 11) & 0x1F) * 255 / 31) as i32;
    let g8 = (((color.rgb565 >> 5) & 0x3F) * 255 / 63) as i32;
    let b8 = ((color.rgb565 & 0x1F) * 255 / 31) as i32;
    
    let gr = (r8 + 255 * 5) / 6;
    let gg = (g8 + 255 * 5) / 6;
    let gb = (b8 + 255 * 5) / 6;
    
    Color::from_888(gr as u16, gg as u16, gb as u16)
}

// Ghost piece lowest position solver
pub fn get_lowest(x: i32, y: i32, t: usize, r: usize, grid: &[[Color; 23]; 12]) -> i32 {
    let cur = TETRAMINOS[t].states[r];
    let mut ys = y;
    loop {
        if ys >= 220 {
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

// Write frozen piece to memory grid
pub fn lock_tetra_in_grid(xt: i32, yt: i32, t: usize, r: usize, grid: &mut [[Color; 23]; 12]) {
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
            let draw_y = y_curr;
            if draw_y >= 0 {
                draw_bevel_square(xl, draw_y, color);
            }
            xl += 10;
        } else if cmd == 2 {
            y_curr += 10;
            xl = xt;
        }
    }
}

// Draw the tetramino on screen dynamically
pub fn draw_tetra_on_screen(xt: i32, yt: i32, t: usize, r: usize) {
    let cur = TETRAMINOS[t].states[r];
    let color = COLORS[TETRAMINOS[t].color_index];
    let mut xl = xt;
    let mut y_curr = yt;
    for &cmd in cur {
        if cmd == 0 {
            xl += 10;
        } else if cmd == 1 || cmd == 3 {
            let draw_y = y_curr;
            if draw_y >= 0 {
                draw_bevel_square(xl, draw_y, color);
            }
            xl += 10;
        } else if cmd == 2 {
            y_curr += 10;
            xl = xt;
        }
    }
}

// Erase the active piece with white
pub fn refresh_tetra(t: usize, px: i32, py: i32, prot: usize) {
    let cur = TETRAMINOS[t].states[prot];
    let mut xl = px;
    let mut y_curr = py;
    for &cmd in cur {
        if cmd == 0 {
            xl += 10;
        } else if cmd == 1 || cmd == 3 {
            let draw_y = y_curr;
            if draw_y >= 0 {
                eadk::display::push_rect_uniform(Rect { x: xl as u16, y: draw_y as u16, width: 10, height: 10 }, COLOR_WHITE);
            }
            xl += 10;
        } else if cmd == 2 {
            y_curr += 10;
            xl = px;
        }
    }
}

pub fn draw_outline_on_screen(xt: i32, yt: i32, t: usize, r: usize) {
    let cur = TETRAMINOS[t].states[r];
    let color = COLORS[TETRAMINOS[t].color_index];
    let ghost_color = get_ghost_color(color);
    let mut xl = xt;
    let mut y_curr = yt;
    for &cmd in cur {
        if cmd == 0 {
            xl += 10;
        } else if cmd == 1 || cmd == 3 {
            let draw_y = y_curr;
            if draw_y >= 0 {
                draw_bevel_square(xl, draw_y, ghost_color);
            }
            xl += 10;
        } else if cmd == 2 {
            y_curr += 10;
            xl = xt;
        }
    }
}

// Main Game loop logic
pub fn run_game(mode: usize, speed: i32, auto_level: bool, resume_state: Option<SavedGameState>) -> GameOutcome {
    let mut x = 140;
    let mut y = 20;
    let mut px;
    let mut py;
    let mut t;
    let mut wait_end_time: Option<u64> = None;
    let mut rot = 0;
    let mut prot;
    let mut plowest = 500;
    
    let mut piece_queue = PieceQueue::new();
    
    let mut hold = -1;
    let mut hold_allowed = true;
    let mut score = 0;
    let mut combo = 0;
    let mut level = speed;
    let mut xp = 0;
    let mut btob = 0;
    let mut lines = 0;
    let mut end = false;
    
    let start_time;
    let mut last_drop_time = eadk::timing::millis();
    let mut coolh_time = eadk::timing::millis();
    let mut arr = true;
    let mut coolr = 0;
    let mut last_move = -1;
    let mut last_save_ms: u64 = 0;
    
    let mut grid = [[COLOR_WHITE; 23]; 12];
    init_grid(&mut grid);

    if let Some(state) = resume_state {
        x = state.x;
        y = state.y;
        rot = state.rot as usize;
        t = state.t as usize;
        hold = state.hold;
        hold_allowed = state.hold_allowed != 0;
        score = state.score;
        combo = state.combo;
        level = state.level;
        xp = state.xp;
        btob = state.btob;
        lines = state.lines;
        
        for i in 0..14 {
            piece_queue.queue[i] = state.queue[i] as usize;
        }
        piece_queue.consumed = state.consumed as usize;
        
        for col in 0..12 {
            for row in 0..23 {
                grid[col][row] = Color { rgb565: state.grid[col * 23 + row] };
            }
        }
        
        start_time = eadk::timing::millis().saturating_sub(state.elapsed_ms);
    } else {
        t = piece_queue.pop();
        start_time = eadk::timing::millis();
    }

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
    
    if hold != -1 {
        draw_tetra_on_screen(30, 60, hold as usize, 0);
    }
    
    eadk::display::draw_string(
        if mode != 1 { "Score :" } else { "Lines :" },
        Point { x: 15, y: 110 },
        false,
        COLOR_WHITE,
        COLOR_BLACK,
    );
    if mode != 1 {
        let score_str = format!("{:07}", score);
        eadk::display::draw_string(&score_str, Point { x: 15, y: 130 }, false, COLOR_WHITE, COLOR_BLACK);
    } else {
        let lines_str = format!("{:02}/40", lines);
        eadk::display::draw_string(&lines_str, Point { x: 15, y: 130 }, false, COLOR_WHITE, COLOR_BLACK);
    }
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
        if resume_state.is_some() {
            let timem = (resume_state.unwrap().elapsed_ms / 60000) as i32;
            let times_val = ((resume_state.unwrap().elapsed_ms % 60000) / 1000) as i32;
            let times = format!("{:02}", times_val);
            let timems_val = ((resume_state.unwrap().elapsed_ms % 1000) / 10) as i32;
            let timems = format!("{:02}", timems_val);
            
            eadk::display::draw_string(&timems, Point { x: 75, y: 180 }, false, COLOR_WHITE, COLOR_BLACK);
            let timem_str = format!("{}", timem);
            eadk::display::draw_string(&timem_str, Point { x: (25 - timem_str.len() as i32 * 10) as u16, y: 180 }, false, COLOR_WHITE, COLOR_BLACK);
            eadk::display::draw_string(&times, Point { x: (65 - times.len() as i32 * 10) as u16, y: 180 }, false, COLOR_WHITE, COLOR_BLACK);
        }
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
                // Always save when the user explicitly exits to menu
                let save_elapsed = eadk::timing::millis() - start_time;
                auto_save_checkpoint(
                    mode, speed, auto_level,
                    x, y, rot as u32, t as u32,
                    hold, hold_allowed,
                    score, combo, level, xp, btob, lines,
                    save_elapsed,
                    &piece_queue,
                    &grid,
                );
                return GameOutcome::Menu;
            } else if choice == GameOutcome::Restart && eadk::input::KeyboardState::scan().key_down(Key::Seven) {
                delete_game_save_file();
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
            // Delete save — game is finished
            delete_game_save_file();
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
        
        // Periodic auto-save every 3 seconds (smart throttle)
        if !end {
            let now_for_save = eadk::timing::millis();
            if now_for_save - last_save_ms >= 3000 {
                let save_elapsed = now_for_save - start_time;
                auto_save_checkpoint(
                    mode, speed, auto_level,
                    x, y, rot as u32, t as u32,
                    hold, hold_allowed,
                    score, combo, level, xp, btob, lines,
                    save_elapsed,
                    &piece_queue,
                    &grid,
                );
                last_save_ms = now_for_save;
            }
        }

        draw_tetra_on_screen(x, y, t, rot);
        eadk::timing::msleep(16);
    }
    
    // Game Over — delete save so "Continue" no longer appears
    // Double-call for robustness
    delete_game_save_file();
    delete_game_save_file();

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
