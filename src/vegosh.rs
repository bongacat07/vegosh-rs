use rapidhash::v3::{DEFAULT_RAPID_SECRETS, rapidhash_v3_seeded};

pub const TABLE_SIZE: usize = 1 << 21;
pub const MASK: u32 = (TABLE_SIZE as u32) - 1;
pub const MAX_KEYS: u32 = 1_000_000;

pub const EMPTY: u8 = 0x00;
pub const OCCUPIED: u8 = 0x01;

pub const KEY_SIZE: usize = 16;
pub const VALUE_SIZE: usize = 32;

#[repr(C, align(64))]
#[derive(Clone, Copy)]
struct Slot {
    key: [u8; KEY_SIZE],
    value: [u8; VALUE_SIZE],
    hash: u64,
    value_len: u8,
    status: u8,
    probe_dist: u16,
    padding: [u8; 4],
}

impl Slot {
    pub const EMPTY: Self = Self {
        key: [0; KEY_SIZE],
        value: [0; VALUE_SIZE],
        hash: 0,
        value_len: 0,
        status: EMPTY,
        probe_dist: 0,
        padding: [0; 4],
    };
}

pub struct Vegosh {
    slots: [Slot; TABLE_SIZE],
    count: u32,
}

impl Vegosh {
    pub const fn new() -> Self {
        Self {
            slots: [Slot::EMPTY; TABLE_SIZE],
            count: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertOutcome {
    Inserted,
    Updated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableFull;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotFound;

#[inline(always)]
fn hash_key(key: &[u8; 16]) -> u64 {
    rapidhash_v3_seeded(key, &DEFAULT_RAPID_SECRETS)
}

#[inline(always)]
pub fn init(table: &mut Vegosh) {
    table.count = 0;
    table.slots.fill(Slot::EMPTY);
}

#[inline(always)]
pub fn insert(
    table: &mut Vegosh,
    key: &[u8; 16],
    value: &[u8; 32],
    value_len: u8,
) -> Result<InsertOutcome, TableFull> {
    assert!((value_len as usize) <= VALUE_SIZE);

    if table.count >= MAX_KEYS {
        return Err(TableFull);
    }

    let hash = hash_key(key);
    let mut index: u32 = (hash as u32) & MASK;

    let mut incoming = Slot {
        key: *key,
        value: *value,
        hash,
        value_len,
        status: OCCUPIED,
        probe_dist: 0,
        padding: [0; 4],
    };

    loop {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            use core::arch::x86_64::{_MM_HINT_T0, _mm_prefetch};

            let next_ptr = table
                .slots
                .as_ptr()
                .add(((index + 2) & MASK) as usize)
                .cast::<i8>();

            _mm_prefetch(next_ptr, _MM_HINT_T0);
        }

        let slot = &mut table.slots[index as usize];

        if slot.status == EMPTY {
            *slot = incoming;
            table.count += 1;
            return Ok(InsertOutcome::Inserted);
        }

        if slot.hash == incoming.hash && slot.key == incoming.key {
            slot.value = incoming.value;
            slot.value_len = incoming.value_len;
            return Ok(InsertOutcome::Updated);
        }

        if incoming.probe_dist > slot.probe_dist {
            std::mem::swap(slot, &mut incoming);
        }

        index = (index + 1) & MASK;
        incoming.probe_dist += 1;
    }
}

#[inline(always)]
pub fn get(table: &Vegosh, key: &[u8; 16]) -> Option<([u8; 32], u8)> {
    let hash = hash_key(key);
    let mut index: u32 = (hash as u32) & MASK;
    let mut probe_dist: u16 = 0;

    loop {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            use core::arch::x86_64::{_MM_HINT_T0, _mm_prefetch};

            let next_ptr = table
                .slots
                .as_ptr()
                .add(((index + 2) & MASK) as usize)
                .cast::<i8>();

            _mm_prefetch(next_ptr, _MM_HINT_T0);
        }

        let slot = &table.slots[index as usize];

        if slot.status == EMPTY {
            return None;
        }

        if slot.hash == hash && slot.key == *key {
            return Some((slot.value, slot.value_len));
        }

        if probe_dist > slot.probe_dist {
            return None;
        }

        index = (index + 1) & MASK;
        probe_dist += 1;
    }
}

#[inline(always)]
<<<<<<< HEAD
pub fn delete_key(table: &mut Vegosh, key: &[u8; 16]) -> i32 {
=======
pub fn delete(table: &mut Vegosh, key: &[u8; 16]) -> Result<(), NotFound> {
>>>>>>> a8a18dd (Modified API to return Option<> and Error Enums)
    let hash = hash_key(key);
    let mut index: u32 = (hash as u32) & MASK;
    let mut probe_dist: u16 = 0;

    loop {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            use core::arch::x86_64::{_MM_HINT_T0, _mm_prefetch};

            let next_ptr = table
                .slots
                .as_ptr()
                .add(((index + 2) & MASK) as usize)
                .cast::<i8>();

            _mm_prefetch(next_ptr, _MM_HINT_T0);
        }

        let slot = &table.slots[index as usize];

        if slot.status == EMPTY {
            return Err(NotFound);
        }

        if slot.hash == hash && slot.key == *key {
            break;
        }

        if slot.probe_dist < probe_dist {
            return Err(NotFound);
        }

        index = (index + 1) & MASK;
        probe_dist += 1;
    }

    loop {
        let next = (index + 1) & MASK;

        if table.slots[next as usize].status == EMPTY || table.slots[next as usize].probe_dist == 0
        {
            table.slots[index as usize] = Slot::EMPTY;
            break;
        }

        table.slots[index as usize] = table.slots[next as usize];
        table.slots[index as usize].probe_dist -= 1;

        index = next;
    }

    table.count -= 1;
    Ok(())
}

#[inline(always)]
pub fn size(table: &Vegosh) -> u32 {
    table.count
}

#[inline(always)]
pub fn clear(table: &mut Vegosh) {
    table.slots.fill(Slot::EMPTY);
    table.count = 0;
}
