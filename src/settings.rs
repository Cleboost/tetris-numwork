#[cfg(target_os = "none")]
use alloc::format;
#[cfg(not(target_os = "none"))]
use std::format;

use crate::eadk::{self, Color, Point, Rect, SCREEN_RECT};
use crate::eadk::input::Key;
use crate::constants::{COLOR_WHITE, COLOR_BLACK};
use crate::drawing::draw_logo;
use crate::storage_lib;

pub fn draw_help_screen() {
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

pub fn draw_settings_layout() {
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

pub fn draw_settings_row(row_idx: usize, is_active: bool, mode: usize, speed: i32, auto_level: bool) {
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

pub fn draw_settings_help(active_row: usize) {
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

pub fn draw_settings_panel(active_row: usize, mode: usize, speed: i32, auto_level: bool) {
    for r in 0..4 {
        draw_settings_row(r, r == active_row, mode, speed, auto_level);
    }
    draw_settings_row(4, active_row == 4, mode, speed, auto_level);
    draw_settings_help(active_row);
}

pub fn draw_main_menu_extras() {
    eadk::display::draw_string("Press [EXE] to start", Point { x: 60, y: 170 }, false, COLOR_WHITE, COLOR_BLACK);
    eadk::display::draw_string("[0] Options", Point { x: 105, y: 190 }, false, COLOR_WHITE, COLOR_BLACK);
}

pub fn save_settings(mode: usize, speed: i32, auto_level: bool) {
    let data = [mode as u8, speed as u8, if auto_level { 1 } else { 0 }];
    storage_lib::storage_file_write("tetris.cfg", &data);
}

pub fn load_settings() -> (usize, i32, bool) {
    let default_settings = (0, 2, true);
    if let Some(data) = storage_lib::storage_extapp_file_read("tetris.cfg") {
        if data.len() >= 3 {
            let mode = data[0] as usize;
            let speed = data[1] as i32;
            let auto_level = data[2] != 0;
            if mode < 3 && speed >= 1 && speed <= 15 {
                return (mode, speed, auto_level);
            }
        }
    }
    default_settings
}

// Menu screen with options
pub fn show_menu() -> (usize, i32, bool) {
    let (mut mode, mut speed, mut auto_level) = load_settings();
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
                    save_settings(mode, speed, auto_level);
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
                save_settings(mode, speed, auto_level);
                menu_page = 0;
                eadk::display::push_rect_uniform(SCREEN_RECT, COLOR_BLACK);
                draw_logo();
                draw_main_menu_extras();
                while eadk::input::KeyboardState::scan().key_down(Key::Back) || eadk::input::KeyboardState::scan().key_down(Key::Zero) {
                    eadk::timing::msleep(10);
                }
            }
        }
        eadk::timing::msleep(16);
    }
    
    (mode, speed, auto_level)
}
