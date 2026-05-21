use crate::eadk::Color;

#[derive(PartialEq, Clone, Copy)]
pub enum GameOutcome {
    Menu,
    Restart,
}

// Custom structure for a Tetramino
pub struct Tetramino {
    pub states: &'static [&'static [u8]],
    pub color_index: usize,
}

pub const COLOR_WHITE: Color = Color::from_888(255, 255, 255);
pub const COLOR_BLACK: Color = Color::from_888(0, 0, 0);
pub const COLOR_GREY: Color = Color::from_888(120, 120, 120);

// Colors for the Tetraminos
pub const COLORS: [Color; 7] = [
    Color::from_888(0, 255, 255),   // Cyan (I)
    Color::from_888(255, 255, 0),   // Yellow (O)
    Color::from_888(200, 0, 255),   // Magenta/Purple (T)
    Color::from_888(255, 150, 0),   // Orange (L)
    Color::from_888(0, 0, 255),     // Blue (J)
    Color::from_888(255, 0, 0),     // Z piece
    Color::from_888(0, 255, 0),     // S piece
];

// Definition of the 7 classic Tetraminos and their rotation states
pub static TETRAMINOS: [Tetramino; 7] = [
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
pub const KICKDATA: [[(i32, i32); 4]; 8] = [
    [(-1, 0), (-1, -1), (0, 2), (-1, 2)],
    [(1, 0), (1, 1), (0, -2), (1, -2)],
    [(1, 0), (1, -1), (0, 2), (1, 2)],
    [(-1, 0), (-1, 1), (0, -2), (-1, -2)],
    [(-2, 0), (1, 0), (-2, 1), (1, -2)],
    [(2, 0), (-1, 0), (2, -1), (-1, 2)],
    [(-1, 0), (2, 0), (-1, -2), (2, 1)],
    [(1, 0), (-2, 0), (1, 2), (-2, -1)],
];

pub const IKICKS: [[usize; 4]; 4] = [
    [0, 5, 0, 7],
    [4, 0, 7, 0],
    [0, 6, 0, 4],
    [6, 0, 5, 0],
];

// Score and levels constants
pub const SCORESYS: [[i32; 5]; 5] = [
    [0, 100, 300, 500, 800],
    [100, 400, 1200, 1600, 0],
    [0, 1200, 1800, 2400, 1200],
    [100, 200, 400, 1200, 0],
    [0, 0, 600, 2400, 0],
];

pub const SCORETITLES: [[&str; 5]; 5] = [
    ["         ", "Single", "Double", "Triple", "Tetris"],
    ["M T-S", "T-S S", "T-S D", "T-S T", ""],
    ["", "T-S B", "T-S D B", "T-S T B", "Tetris B"],
    ["M T-S", "M T-S S", "M T-S D", "T-S T", ""],
    ["", "", "T-S M D B", "T-S T B", ""],
];

pub const BSYS: [[i32; 5]; 5] = [
    [0, 0, 0, 0, 1],
    [1, 1, 1, 1, 0],
    [1, 1, 1, 1, 1],
    [0, 0, 1, 1, 0],
    [0, 0, 1, 1, 0],
];

pub const LEVELSYS: [i32; 5] = [0, 1, 3, 5, 8];

// Main Logo and background drawing
pub const LOGO: [[u8; 21]; 5] = [
    [1, 1, 1, 0, 2, 2, 2, 0, 3, 3, 3, 0, 4, 4, 4, 0, 5, 0, 0, 6, 6],
    [0, 1, 0, 0, 2, 0, 0, 0, 0, 3, 0, 0, 4, 0, 4, 0, 0, 0, 6, 0, 0],
    [0, 1, 0, 0, 2, 2, 0, 0, 0, 3, 0, 0, 4, 4, 0, 0, 5, 0, 0, 6, 0],
    [0, 1, 0, 0, 2, 0, 0, 0, 0, 3, 0, 0, 4, 0, 4, 0, 5, 0, 0, 0, 6],
    [0, 1, 0, 0, 2, 2, 2, 0, 0, 3, 0, 0, 4, 0, 4, 0, 5, 0, 6, 6, 0],
];

pub const LOGO_COLORS: [Color; 7] = [
    Color::from_888(0, 0, 100),
    Color::from_888(255, 0, 0),
    Color::from_888(255, 150, 0),
    Color::from_888(255, 255, 0),
    Color::from_888(0, 255, 0),
    Color::from_888(0, 255, 255),
    Color::from_888(255, 0, 255),
];
