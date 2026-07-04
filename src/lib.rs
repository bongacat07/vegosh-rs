pub mod vegosh;

pub use crate::vegosh::Vegosh;
pub use crate::vegosh::{
    KEY_SIZE, MAX_KEYS, TABLE_SIZE, VALUE_SIZE, clear, delete_key, get, init, insert, size,
};
