mod vegosh;

use vegosh::{Vegosh, clear, delete_key, get, init, insert, size};

pub const TABLE_SIZE: usize = 1 << 21;
pub const MASK: u32 = (TABLE_SIZE as u32) - 1;
pub const MAX_KEYS: u32 = 1_000_000;

pub const EMPTY: u8 = 0x00;
pub const OCCUPIED: u8 = 0x01;

pub const KEY_SIZE: usize = 16;
pub const VALUE_SIZE: usize = 32;
