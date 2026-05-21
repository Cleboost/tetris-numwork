static mut RNG_STATE: u64 = 123456789;

pub fn seed_rng(seed: u64) {
    unsafe {
        RNG_STATE = if seed == 0 { 123456789 } else { seed };
    }
}

pub fn next_random() -> u32 {
    unsafe {
        RNG_STATE = RNG_STATE.wrapping_mul(1664525).wrapping_add(1013904223);
        (RNG_STATE >> 16) as u32
    }
}

// 7-bag randomizer
pub fn random_bag() -> [usize; 7] {
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
