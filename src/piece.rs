use crate::eadk::{self, Rect};
use crate::constants::COLOR_BLACK;
use crate::rng::random_bag;
use crate::game::draw_tetra_on_screen;

// Next Piece Queue management
pub struct PieceQueue {
    pub queue: [usize; 14],
    pub consumed: usize,
}

impl PieceQueue {
    pub fn new() -> Self {
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

    pub fn pop(&mut self) -> usize {
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

    pub fn draw_preview(&self) {
        eadk::display::push_rect_uniform(Rect { x: 250, y: 50, width: 40, height: 130 }, COLOR_BLACK);
        for ns in 0..4 {
            draw_tetra_on_screen(250, 60 + ns as i32 * 30, self.queue[ns], 0);
        }
    }
}
