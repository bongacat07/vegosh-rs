# Vegosh

A fixed capacity, open-addressing hash table using Robin Hood hashing with
backward-shift deletion. Built for a bounded, known-size key space. **NOT** a
general-purpose growable map.

Design philosophy heavily inspired by [TigerStyle](https://tigerstyle.dev/)
(TigerBeetle). Used static allocation, explicit limits, and asserted invariants
over dynamic, unbounded behavior. 

Style goals in order: **safety,performance, developer experience.**

## What this is

- **Robin Hood open addressing** — minimizes probe sequence variance by
  letting "richer" entries (short probe distance) yield their slot to
  "poorer" ones (long probe distance) during insertion.
- **Backward-shift deletion** — no tombstones. Deleting a key shifts
  subsequent entries backward to close the gap, so probe sequences never
  degrade over the table's lifetime the way tombstone-based schemes do.
- **Fixed capacity, static allocation** — `TABLE_SIZE = 2^21` slots,
  `MAX_KEYS = 1_000_000`, giving a fixed 50% max load factor. No resizing,
  no reallocation, no growth path. The table's memory footprint (128 MiB)
  is fully known at compile time.
- **Fixed-width keys/values** — 16-byte keys, 32-byte values. Not a general
  string/blob map; built for fixed-size identifiers (hashes, UUIDs, etc).
- **Cache-conscious layout** — each `Slot` is exactly 64 bytes, aligned to
  a cache line, with the hash stored inline to avoid recomputation on
  lookup and to allow early rejection before a key comparison.
- **Software prefetching** on `x86_64` to hide cache-miss latency during
  longer probe sequences.

## Why fixed capacity, on purpose

In the spirit of TigerStyle's "put a limit on everything because
everything has a limit": this table does not grow. `MAX_KEYS` is a hard
ceiling, checked explicitly on insert, and the 50% load factor is what
keeps expected probe distances small and bounded. This is a deliberate
trade for predictability. The caller must know their key cardinality
upper bound ahead of time. There is no amortized-growth story here, and
there isn't meant to be one.

## Correctness invariants

The current implementation relies on a few invariants that are enforced by
construction rather than checked defensively at runtime:

- The 50% load factor guarantees an empty slot always exists before
  `MAX_KEYS` is reached, so the probe loop in `insert` is guaranteed to
  terminate.
- `probe_dist: u16` is never expected to overflow at this load factor, but
  this isn't currently asserted.

Per TigerStyle's assertion philosophy, assertions exist to catch
programmer error, not expected runtime conditions, and the correct
response to a violated invariant is to crash immediately rather than limp
along with corrupted state.  A debug build should assert these invariants
explicitly rather than relying on them silently holding. This is a known
gap

## Thread safety

**Not thread-safe.** There is no internal synchronization. `insert` and
`delete_key` take `&mut Vegosh`; `get` takes `&Vegosh`. Rust's borrow
checker enforces exclusivity for a normal reference, but this table is
intended to live behind a `static`, which requires the caller to choose
and implement their own synchronization strategy (e.g. a `Mutex<Vegosh>`,
sharding across threads, or one table per thread). This is left
unimplemented deliberately rather than baked in, so callers can pick a
strategy that fits their concurrency model instead of paying for one they
don't need.

## Usage

```rust
static TABLE: Vegosh = Vegosh::new();
```

<<<<<<< HEAD
Do not construct this as a bare local (`let table = Vegosh::new()`). At
128 MiB it will overflow a typical thread stack. Use `static`, `Box::new`,
or a heap-backed lazy static.
=======
### Initializing

A newly constructed table is already empty, so calling `init()` is only
necessary when reusing an existing table.

```rust
use vegosh::{init, Vegosh};

static mut TABLE: Vegosh = Vegosh::new();

unsafe {
    init(&mut TABLE);
}
```

### Inserting

Keys are fixed at 16 bytes and values at 32 bytes. value_len specifies how
many bytes of the value are logically valid.
```rust
use vegosh::{insert, InsertOutcome, TableFull, Vegosh};

static mut TABLE: Vegosh = Vegosh::new();

let key = *b"0123456789abcdef";

let mut value = [0u8; 32];
value[..11].copy_from_slice(b"hello world");

unsafe {
    match insert(&mut TABLE, &key, &value, 11) {
        Ok(InsertOutcome::Inserted) => println!("inserted"),
        Ok(InsertOutcome::Updated) => println!("updated existing key"),
        Err(TableFull) => println!("table is full"),
    }
}
```
### Looking up a value

```rust
use vegosh::{get, Vegosh};

static mut TABLE: Vegosh = Vegosh::new();

let key = *b"0123456789abcdef";

unsafe {
    match get(&TABLE, &key) {
        Some((value, len)) => {
            println!("{}", String::from_utf8_lossy(&value[..len as usize]));
        }
        None => println!("key not found"),
    }
}
```

### Removing a key

```rust
use vegosh::{delete, NotFound, Vegosh};

static mut TABLE: Vegosh = Vegosh::new();

let key = *b"0123456789abcdef";

unsafe {
    match delete(&mut TABLE, &key) {
        Ok(()) => println!("removed"),
        Err(NotFound) => println!("key not found"),
    }
}
```

### Clearing the table

```rust
use vegosh::{clear, Vegosh};

static mut TABLE: Vegosh = Vegosh::new();

unsafe {
    clear(&mut TABLE);
}
```

### Querying the number of entries

```rust
use vegosh::{size, Vegosh};

static mut TABLE: Vegosh = Vegosh::new();

unsafe {
    println!("entries: {}", size(&TABLE));
}
```
>>>>>>> a8a18dd (Modified API to return Option<> and Error Enums)

## API

| Function | Description |
|---|---|
| `init(table)` | Reset a table to empty. |
| `insert(table, key, value, value_len)` | Insert or update. Returns `0` on insert, `1` on update, `-1` if `MAX_KEYS` reached. |
| `get(table, key, out_value, out_value_len)` | Lookup. Returns `0` on hit, `-1` on miss. |
| `delete_key(table, key)` | Remove a key. Returns `0` on success, `-1` if not found. |
| `size(table)` | Current key count. |
| `clear(table)` | Reset a table to empty (same as `init`). |

## Non-goals

- Dynamic resizing.
- Variable-length keys.
- Thread safety out of the box.
- General-purpose map semantics (iteration, entry API, etc.).

## Attribution

Design philosophy inspired by [TigerStyle](https://tigerstyle.dev/), the
engineering style guide developed by [TigerBeetle](https://github.com/tigerbeetle/tigerbeetle),
itself inspired by NASA's *Power of Ten Rules for Safety-Critical Code*.
