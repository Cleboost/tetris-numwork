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
pub mod storage_lib;
pub mod constants;
pub mod rng;
pub mod drawing;
pub mod piece;
pub mod settings;
pub mod game;

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

use crate::rng::seed_rng;
use crate::settings::show_menu;
use crate::game::run_game;
use crate::constants::GameOutcome;

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
