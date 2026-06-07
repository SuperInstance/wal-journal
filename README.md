# wal-journal

A Write-Ahead Log implementation with checkpointing and crash recovery.

## Features

- Sequential log entry append
- Checkpointing for faster recovery
- Sequence-numbered entries for ordering
- Crash recovery via log replay
- Truncation of checkpointed entries
- Zero external dependencies

## Usage

```rust
use wal_journal::Wal;

let mut wal = Wal::new();
wal.append(b"transaction data");
wal.checkpoint();
let entries = wal.recover();
```

## Modules

- `entry` — Log entry structure with sequence numbers
- `log` — Core WAL append and read operations
- `checkpoint` — Checkpoint management
- `recovery` — Crash recovery and replay
- `sequence` — Monotonic sequence number generation
