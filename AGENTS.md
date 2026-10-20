# agents.md — Music Player Firmware

## Project Summary

Embedded Rust portable audio player built on the **STM32H7A3VI** microcontroller using the
**Embassy** async runtime. Audio files are read from an **exFAT** SD card, decoded by
**Symphonia** (MP3 and FLAC), and streamed as PCM over **SAI** to an **AK4377**
DAC/headphone amplifier chip which handles digital-to-analogue conversion and drives
the headphones.

The filesystem layer is **exfat-slim 0.7.0** (pinned, `no_std`), used directly inside
`app-core` as the shared FS for the desktop simulator and the firmware — host and target
differ only in their `BlockDevice` implementation. The `FileSystem` has exactly one
owner: the `FileManager` handler inside `file_manager_task` (M4). Music metadata lives in
on-card database files (`MUSICLIB.CAT` catalog + `MCOMP.CAT` interned-component table +
per-field sorted views `TITLEASC.LST` / `ARTISTASC.LST`), so RAM usage is O(1) in library
size and sorting never repeats for an unchanged card. See **SD Library & On-Card
Catalog** for the full design.

The UI, player, and file-access logic communicate via an **actor-model event system**
using `heapless::Vec<Event, 8>` internal queues, formalised by the `EventHandler`
trait. A static Embassy `PubSubChannel` bridges these queues between async tasks —
this is no longer just planned: the desktop simulator runs the real Embassy executor
(`platform-std`) with `ui_task` and `player_task` spawned from
`#[embassy_executor::main]` (plus `file_manager_task` in M4), so the task/channel
design is validated on the host before firmware bringup. The same design carries over
to the STM32 firmware tasks.

A desktop **mock** crate mirrors the embedded display using `embedded-graphics-simulator`
so UI code can be developed and tested without hardware.

### Current status

- **Done:** retained-mode GUI framework, `Player` + `Playlist` state machines (actor-model
  pattern, `EventHandler` trait, event queues), transport controls + progress bar UI,
  `app-mock` simulator running the real Embassy executor with a static `PubSubChannel`
  bridging `ui_task` / `player_task`. `app-hal` dissolved — its traits live in
  `app-core/src/hal.rs` (1ff5737). `app-core` is `no_std` with `extern crate alloc`
  (1d28ce7). `Playlist` is a `heapless::Deque<Track, MAX_TRACKS=16>` with `current_index`
  (2a3459a, dffeb3a). `Track` is `{title, artist: String<256>, duration_ms: u32,
  index: usize}`. `State::TrackChanged(Option<Box<Track>>)` keeps the `Event` enum small
  (a4dc9bf). The M4 event plumbing exists end-to-end: `Command::UpdatePlaylist(
  Box<Playlist>)` (0cce3c2, 8b2b52b), `Event::FileManager(Fetch::Playlist { sort_order,
  start_index, end_index })` with `SortOrder` = Title/Artist × Asc/Desc (a5f23d2,
  3c230fa), and a `FileManager` handler skeleton in `app-core/src/file_manager.rs`
  (3e45ab6) — `fetch_playlist`/`fetch_track`/`get_list_filename` are stubs.
- **In flight — branch `sorted-list-implementation`, build currently broken:** work on
  the M4 fetch path is under way (stages in **Milestones**). The only compile error is
  `file_manager.rs:48` — `get_list_filename(sort_order) -> string` names the `string`
  module instead of a type (E0573); Stage 0 fixes it (`-> &'static str`).
- **Decided for M4 (locked design, being implemented):** exFAT-only storage via
  exfat-slim 0.7.0 with `default-features = false` (no alloc, no logging, **no embassy**
  — the crate's built-in fs actor was evaluated and rejected; see *exfat-slim notes*).
  **`file_manager_task` is the FS actor**: its `FileManager` handler is the sole owner
  of the raw `exfat_slim::asynchronous::FileSystem`, so "streaming a music file while
  opening a sorted list" is one task interleaving at await points — no FS sharing, no
  locks. `EventHandler` gains `async fn handle_event_async` (default = the sync
  `handle_event`). On-card database files with an FNV-1a generation fingerprint scanned
  every boot; `Track.index` becomes `cat_index: u32` pointing at its `.CAT` record
  (string interning: the record's 16-deep `u32` chain references directory/filename
  strings in `MCOMP.CAT`). The `Playlist` window is externally driven: `Player` emits
  `Fetch::Playlist`, `file_manager_task` awaits `fetch_window(order, start, end)` and
  publishes `UpdatePlaylist`; `Player` splices on apply and prefetches near the window
  edge (boot fetch `[0,16)` + in-playback prefetch). The simulator mounts a committed
  gzipped exFAT image in RAM and populates it through exfat-slim's own write path (CJK
  samples included). See **SD Library & On-Card Catalog**, **Event Architecture**, and
  **Milestones**.
- **Not started:** Symphonia decoding (`app-core/src/decoder.rs`), `memory.x`, CI pipeline,
  and all hardware bringup (the `app-firmware` crate is still a hello-world placeholder
  with no dependencies and does not yet build for the embedded target).

---

## Hardware

| Component | Part | Notes |
|---|---|---|
| Microcontroller | STM32H7A3VI (LQFP100) | Cortex-M7 @ 280MHz, 2MB Flash, 1.4MB SRAM |
| Display | TBD (monochrome) | 240×240 is a placeholder; Gray8 UI; framebuffer provisionally budgeted (see Memory Layout) |
| DAC / Amp | AK4377AECB | Controlled via I2C; audio streamed via SAI (I2S-compatible) |
| Storage | SD card (exFAT) | Connected via SDMMC peripheral (4-bit mode, 3.3V) |
| Debug probe | ST-LINK V3 / CMSIS-DAP | SWD interface, RTT logging via probe-rs |
| Package | LQFP100 | 80 GPIOs — chosen over LQFP64 for pin budget headroom |

### Key hardware decisions recorded

- **STM32H7A3VI chosen over STM32H753** — more SRAM (1.4MB vs ~1MB), lower clock (280MHz)
  is not a concern for audio decode workloads (~50–200 DMIPS required vs 599 available.
- **LQFP100 chosen over LQFP64** — LQFP64 has only 49 GPIOs which is insufficient when
  SDMMC, SAI, I2C, SPI, USB, debug, and UI GPIOs are all counted.
- **Single SAI interface is sufficient** — only one audio output destination (AK4377).
- **Internal 2MB Flash used** — no external QSPI Flash needed; firmware fits comfortably.
- **Standard 3.3V SD card** — VDDMMC pin not needed; SDMMC powered from main VDD rail.
- **Single-core MCU** — dual-core STM32H745/H747 rejected; audio decode is bursty not
  continuous, DMA + Embassy async handles producer/consumer without a second core.
- **exFAT-only storage (M4 decision)** — all cards are prepared as exFAT (any size card
  can be reformatted; exFAT is the native format for ≥64GB SDXC). One FS layer
  (exfat-slim) instead of dual FAT32+exFAT support; swapping or adding an FS later is
  contained to `library.rs` and the `BlockDevice` adapters.

### AK4377 initialisation notes

Configure over I2C before streaming audio:
- Release PDN pin (pull high) before any I2C communication
- Set **ACKS** bit → slave I2S mode (STM32 is SAI master)
- Set **DIF[1:0]** = `01` → standard I2S format
- Set **FS[3:0]** to match sample rate
- Enable **PMDA** (DAC), **PMCP** (charge pump)
- Set volume via **ATL / ATR** registers

---

## Cargo Workspace Structure

Note: `memory.x` and `.gitlab-ci.yml` are **planned but do not exist yet**.

```
music-player-firmware/
├── Cargo.toml                      # Workspace manifest, shared dependency versions
│                                   #   exfat-slim = "0.7" (replaces the old embedded-sdmmc pin)
├── rust-toolchain.toml             # Pinned nightly toolchain (nightly-2026-04-11)
├── memory.x                        # (planned, not yet created) linker memory regions
├── .cargo/
│   └── config.toml                 # `[target.thumbv7em-none-eabihf]` runner + rustflags only
│                                   # NO [build] target here — see app-firmware below
├── tools/
│   └── make_exfat_image.sh          # (M4, planned) one-time: mkfs.exfat a small volume,
│                                   #   gzip it → app-mock/assets/exfat.img.gz
│
├── app-core/                       # Pure business logic — no hardware, no_std + alloc
│   └── src/
│       ├── lib.rs                  # Module declarations (#![no_std], extern crate alloc)
│       ├── event.rs                # Button, Command, State, Screen, Fetch, Event enums +
│       │                           #   EventHandler trait (+ M4: handle_event_async),
│       │                           #   EventQueue type
│       ├── player.rs               # Player handler — implements EventHandler, owns state
│       ├── player/
│       │   └── tests.rs            # Player pure-logic tests (submodule pattern)
│       ├── playlist.rs             # Deque<Track,16> + current_index today; (M4) gains
│       │                           #   window_start + splice methods (window is fed
│       │                           #   externally by file_manager_task)
│       ├── playlist/
│       │   └── tests.rs           # Playlist pure-logic tests
│       ├── track.rs                # {title, artist: String<256>, duration_ms: u32,
│       │                           #   index: usize}; (M4) index → cat_index: u32
│       ├── file_manager.rs         # FileManager handler (skeleton) — (M4) sole owner
│       │                           #   of the raw FileSystem; fetch_playlist /
│       │                           #   fetch_track / get_list_filename
│       ├── hal.rs                  # AudioFileReader, AudioOutput traits + error enums
│       │                           #   (M4 decision: NO Filesystem trait — exfat-slim IS
│       │                           #    the shared FS layer, used directly in library.rs)
│       ├── library.rs              # (M4, planned) SortOrder, FNV-1a fingerprint DFS,
│       │                           #   catalog/MCOMP rebuild, LST load-or-build,
│       │                           #   fetch_window(order, start, end) — raw async API,
│       │                           #   fns take &mut FileSystem
│       ├── library/
│       │   └── tests.rs            # Pure-logic tests: record/table/header codecs, FNV
│       │                           #   vectors, title derivation, extension filter,
│       │                           #   duration estimator, SortOrder comparator,
│       │                           #   chain→path reconstruction
│       └── ui/
│           ├── ui.rs               # Widgets live here: Label, ProgressBar, PlayButton,
│           │                       #   HorizontalLine, List, clear_background, fmt_time_ms
│           ├── display_config.rs   # DISPLAY_WIDTH/HEIGHT (240×240), PIXEL_SCALE (3)
│           ├── screen_names.rs     # ScreenName enum (Main, Settings)
│           ├── screen_manager.rs   # EventHandler impl routing events to active screen
│           └── screens/            # Screen structs (MainScreen, SettingsScreen), each
│               │                   #   an EventHandler with its own event queue
│               ├── main_screen.rs
│               └── settings_screen.rs
│
├── app-firmware/                   # Embassy firmware — STM32 target only
│   ├── .cargo/config.toml          # [build] target = "thumbv7em-none-eabihf" ONLY HERE
│   └── src/
│       ├── main.rs                 # (placeholder hello-world — no dependencies yet)
│       └── drivers/sd_card.rs      # (M9, planned) exfat-slim BlockDevice over embassy-stm32
│                                   #   SDMMC + MBR type-0x07 partition-offset translation
│
└── app-mock/                       # Desktop simulator — std, runs real Embassy executor
    ├── Cargo.toml                  # edition = "2024"; + exfat-slim, miniz_oxide (gunzip)
    ├── assets/
    │   ├── exfat.img.gz            # (M4, planned) committed empty exFAT volume,
    │   │                           #   ~tens of KB, regenerated via tools/make_exfat_image.sh
    │   └── music/                  # (M4, planned) sample files copied into the RAM image
    │                               #   at startup — CJK-named samples included
    └── src/
        ├── main.rs                 # #[embassy_executor::main], static PubSubChannel,
        │                           #   spawns ui_task + player_task + (M4)
        │                           #   file_manager_task — spawn order matters, see
        │                           #   Event Architecture
        ├── tasks.rs                # Task<T: EventHandler> struct (owns handler + pub/sub
        │                           #   endpoints; M4: awaits handle_event_async),
        │                           #   EventChannel (M4: 3 pub / 3 sub)
        ├── tasks/
        │   ├── player_task.rs      # Player task — services Player commands + 100ms tick;
        │   │                       #   (M4) Player::new(empty Playlist), windows arrive
        │   │                       #   via UpdatePlaylist
        │   ├── ui_task.rs          # UI task — keyboard input, screen refresh loop
        │   └── file_manager_task.rs # (M4, planned) THE FS ACTOR TASK — constructs the
        │                           #   device + FileSystem, mount → Library::open_or_build
        │                           #   → boot fetch [0,16) → publish → loop on
        │                           #   Event::FileManager(_)
        ├── mocks.rs                # mock_display, mock_window, mock_player constructors
        ├── mocks/
        │   ├── mock_display.rs
        │   ├── mock_player.rs      # (M4) synthetic 2-track fallback when assets are
        │   │                       #   missing; the real library comes from the RAM
        │   │                       #   image via file_manager_task
        │   └── sd_image.rs         # (M4, planned) ExFatRamBlockDevice (gunzip→Vec,
        │                           #   impl async BlockDevice<512>, Align = A4) +
        │                           #   populate_from_dir() via exfat-slim
        │                           #   create_directory/write
        └── tests/                   # (M4, planned) behavioral suite, #[cfg(test)] — the
                                    #   image tooling is simulator-only, so all tests that
                                    #   need a mounted volume live here: scan→CAT, LST
                                    #   ordering (asc + desc), fingerprint change/stale,
                                    #   interning dedup, chain→path reconstruction, window
                                    #   slides, player e2e with prefetch during playback,
                                    #   reopen-by-CJK-path
```

---

## Architecture Rules

1. **Hardware code lives only in `app-firmware/src/drivers/` and `app-firmware/src/tasks/`.**
   No Embassy types, no STM32 peripheral types anywhere in `app-core`.

2. **`app-core` must compile with `cargo test` on the host** (std enabled).
   exfat-slim is `no_std` (allocator optional — the core paths we use, directory
   iteration with a caller-supplied buffer and file read/write, are allocation-free),
   so it may be used directly in `app-core`. Never add `embassy-*`, `cortex-m`, or any
   host-only dep to `app-core`.

3. **All drawing code uses `DrawTarget<Color = Gray8>`** — never reference
   `SimulatorDisplay` directly outside of `app-mock/src/mocks/mock_display.rs`
   (where the `MockSimulatorDisplay` alias is defined). This keeps UI code
   portable to the real display driver with zero changes.

4. **PCM DMA buffers go in AXISRAM** via `#[link_section = ".axisram"]`. Do not place
   DMA buffers in DTCM — the DMA controller cannot access it.

5. **Use `defmt` for all firmware logging** — never `println!` or the `log` crate in
   `app-firmware`. Use `#[derive(defmt::Format)]` instead of manual `Debug` impls.
   exfat-slim's logging features (`log`/`defmt`) stay **off** in app-core; firmware
   may enable `defmt` at M7+.

6. **Symphonia requires a global allocator** — `embedded-alloc` is used. Budget ~64KB
   for the heap. FLAC and MP3 are safe; AAC/Opus are marginal at this heap size.

7. **Actor model for UI/Player communication.** `Player` owns its state
   exclusively. Other modules send commands via `Event::Player(...)` and
   receive state changes via events emitted into a bounded `heapless::Vec`.
   No module accesses `Player` fields directly.

8. **Unit tests live in sibling submodules, never inline.** Place tests in
   `src/<module>/tests.rs` and declare them with `#[cfg(test)] mod tests;`
   inside `src/<module>.rs`. This keeps source files readable while retaining
   `use super::*` access to private items. Tests that need a mounted volume
   (anything touching the exFAT image) live in `app-mock/src/tests/` — the
   image tooling is simulator-only, and `app-core` must never depend on
   `app-mock`.

9. **exfat-slim is the shared FS layer, used directly in `app-core`; the `FileSystem`
   has exactly one owner; `BlockDevice` adapters live at the edges.** Every exfat-slim
   file operation takes `&mut FileSystem` (`File::read(&mut self, fs, buf)`, `seek`,
   `write`), so single ownership is enforced by the type system: the `FileManager`
   handler (`app-core/src/file_manager.rs`) owns the raw
   `exfat_slim::asynchronous::FileSystem` and **`file_manager_task` is the FS actor** —
   no other task touches the FS. `library.rs` calls the raw async API (`open` with
   `OpenOptions`, `File::read/seek/write` taking `&mut FileSystem`, `read_dir` with a
   caller-supplied 765B name buffer). The crate's `embassy` feature (internal
   request-queue actor) is **rejected** — see *exfat-slim notes*. `app-mock` supplies
   `ExFatRamBlockDevice` (gunzipped committed image); `app-firmware` supplies the
   embassy-stm32 SDMMC adapter **plus MBR partition-offset translation** (exfat-slim
   mounts at view-sector 0; real SDXC cards carry an MBR with a type-0x07 exFAT
   partition whose LBA the adapter must add to every sector address — M9).
   `hal.rs` keeps `AudioFileReader`/`AudioOutput` as the M5 decoder's ports.
   (Supersedes the earlier embedded-sdmmc/FAT32 plan and the `Filesystem` trait.)

10. **The on-card database files are private formats; the fingerprint is the glue.**
    Any byte layout change to `MUSICLIB.CAT` / `MCOMP.CAT` / the `*.LST` views bumps its
    `version` field (old files auto-rebuild). All files of one generation share
    one fingerprint; never mix generations.

11. **`handle_event_async` is the async entry point; sync `handle_event` is the
    default.** Handlers needing `await` (FileManager's FS reads today, the M5 Decoder
    later) override `async fn handle_event_async`; `Task::service_event` awaits it.
    Native async-fn-in-trait (AFIT) is fine on the pinned nightly and in `no_std`
    (exfat-slim itself uses AFIT for its `BlockDevice` trait). Handlers without async
    needs (Player, screens) inherit the default that delegates to the sync method.

12. **All card reads route through `file_manager_task` — including the decoder's.**
    The M5 decode task streams its music file by requesting reads from the FS actor
    task; it never owns the `FileSystem`, and the `FileSystem` is never shared behind a
    mutex or moved between tasks. The request mechanism (zero-copy lending channel vs
    event-based chunks) is decided at M5; the constraint is fixed now.

---

## Cargo Configuration — Critical Details

### Workspace `.cargo/config.toml`
```toml
# Only the runner and rustflags — NO [build] target
[target.thumbv7em-none-eabihf]
runner    = "probe-rs run --chip STM32H7A3VITx"
rustflags = [
  "-C", "linker=flip-link",
  "-C", "link-arg=-Tlink.x",
  "-C", "link-arg=-Tdefmt.x",
]
```

### `app-firmware/.cargo/config.toml`
```toml
# Embedded target set here only, so app-mock and tests use the host target
[build]
target = "thumbv7em-none-eabihf"
```

> **Why this split matters:** Setting `[build] target` at the workspace level causes
> `app-mock` to compile against `thumbv7em-none-eabihf` (a `no_std` target), which
> produces `error[E0463]: can't find crate for std`. Keeping it in `app-firmware/.cargo/`
> scopes it correctly.

### `rust-toolchain.toml`
```toml
[toolchain]
channel    = "nightly-2026-04-11"   # update this pin periodically
targets    = ["thumbv7em-none-eabihf"]
components = ["rust-src", "llvm-tools-preview", "rustfmt", "clippy"]
```

### Edition
All crates use **`edition = "2024"`** (stabilised in Rust 1.85, February 2025).

### Key dependency notes (M4)
- `exfat-slim = "0.7"` — pinned pre-1.0, single-author crate; accepted risk
  (see *SD Library & On-Card Catalog* → *exfat-slim notes*). `default = []` and it
  stays that way: no logging, no alloc, no embassy. The `embassy` feature
  (crate-internal request-queue actor: `fs_actor_task` + `FileHandle` client API;
  implies `alloc`, `Vec`-per-read copies) was **evaluated and rejected** —
  `file_manager_task` owns the raw `FileSystem` instead (rule 9). With all features
  off the dep tree is `aligned`, `bitflags`, `block-device-driver`, `byteorder`,
  `exfat-slim-macros`, `heapless`, `thiserror` — all `no_std`, zero `embassy-*`
  crates in `app-core`.
- `miniz_oxide` in `app-mock` only — gunzips `assets/exfat.img.gz` at startup.
- The unused `embedded-sdmmc` workspace pin is removed in M4 Stage 0 (still present in
  the manifest today, referenced by no crate).

---

## Memory Layout (STM32H7A3VI)

```
FLASH   (rx)  : ORIGIN = 0x08000000, LENGTH = 2M    # firmware code + rodata
ITCM    (rwx) : ORIGIN = 0x00000000, LENGTH = 64K   # time-critical ISR code
DTCM    (rw)  : ORIGIN = 0x20000000, LENGTH = 128K  # stack, Embassy runtime
AXISRAM (rw)  : ORIGIN = 0x24000000, LENGTH = 1024K # heap, PCM DMA buffers
SRAM1   (rw)  : ORIGIN = 0x30000000, LENGTH = 64K
SRAM2   (rw)  : ORIGIN = 0x30010000, LENGTH = 64K
```

Approximate runtime memory budget:

| Region | Allocation | Size |
|---|---|---|
| AXISRAM | Symphonia heap (`embedded-alloc`) | 64KB |
| AXISRAM | PCM double-buffer (DMA) | ~32KB |
| DTCM | Embassy + task stacks | ~64KB |
| SRAM1 | Display framebuffer — 240×240 Gray8 full-frame (**provisional**, display not yet chosen) | ~57.6KB |
| AXISRAM | `FileManager`/`Library` state + `FileSystem` slot cache (N=8 × 512B) + `Deque<Track,16>` window (16 × ~520B) | ~13KB |
| AXISRAM | LST build sort buffer `Vec<Track, CAP=256>` (~520B/entry) | ~148KB, boot-transient only |
| AXISRAM | DFS walk buffers (765B name buf, ~4KB path prefix, on the file_manager_task stack) | ~5KB, boot-transient only |

**Display framebuffer (provisional estimate):** a full 240×240 Gray8 frame is
57.6KB — the safe upper bound matching the `DrawTarget<Color = Gray8>` rule, so the
UI code's colour model carries to firmware unchanged. SRAM1 (64KB, otherwise
unbudgeted) hosts it with ~6KB spare; AXISRAM is the fallback (steady-state usage
is ~109KB of 1024KB). Alternatives once the panel is chosen: a 1bpp buffer (7.2KB,
needs draw-time or flush-time conversion) or line/window streaming (~0 RAM,
controller-dependent). No double-buffer — the retained-mode UI redraws on event
only. Final placement also depends on the display controller's DMA paths (DTCM is
excluded for DMA, rule 4; SRAM1/AXISRAM are both reachable).

The static `PubSubChannel<Event, 64>` stays small: `State::TrackChanged(Option<Box<Track>>)`
and `Command::UpdatePlaylist(Box<Playlist>)` box the big payloads, so each slot is a few
dozen bytes.

---

## SD Library & On-Card Catalog (M4 design — locked)

RAM usage is O(1) in library size: the sorted lists live **on the SD card**; the
in-memory `Library` is a ~40-byte accessor. Four private files in the volume root
(`MUSICLIB.CAT`, `MCOMP.CAT`, `TITLEASC.LST`, `ARTISTASC.LST`), all rebuilt together
(one fingerprint, one generation):

### `/MUSICLIB.CAT` — unsorted catalog (scan order)

Header **24 B**: 4B magic `"MLIB"` | 4B version (u32 LE) |
4B count (u32 LE) | 8B fingerprint (u64 LE) | 4B skipped_count (u32 LE).

Records **580 B** each (record *i* at `24 + i×580`; truncation check
`24 + count×580 == file_len`):

| Offset | Size | Field | Encoding |
|---|---|---|---|
| 0 | 256 | `title` | UTF-8 NUL-padded — filename stem (CJK intact); M5 overwrites with parsed tags |
| 256 | 256 | `artist` | UTF-8 NUL-padded — `""` until M5 |
| 512 | 64 | `chain[16]` | u32 LE each, `0xFFFFFFFF`-padded — interned component indices, root→leaf; last element = the filename's `MCOMP.CAT` index |
| 576 | 4 | `duration_ms` | u32 LE — size-based estimate at scan (~192 kbps MP3, ~900 kbps FLAC) until M5 |

### `/MCOMP.CAT` — interned component table

Header **24 B**: `magic "MCMP"` | version | count | fingerprint (== CAT's) | reserved.
Body: **765 B** entries (entry *i* at `24 + i×765`), UTF-8 NUL-padded component names —
directories **and** filenames interned here. 765 B is the exact exFAT component
maximum (255 UTF-16 units × 3), matching exfat-slim's own `MAX_NAME_LEN`; a full-width
name has no NUL (decode = bytes up to first NUL, else all 765). Directory interning is
free during the rebuild DFS (each directory is visited exactly once — index assigned on
entry); filename entries are appended one per file (~unique anyway).

### `/TITLEASC.LST` + `/ARTISTASC.LST` — sorted views (u32 catalog indices)

One file per sort field, header **24 B** each: `magic "MLST"` | version | order
(u32 LE, canonical base order of the file: 0 = TitleAsc, 1 = ArtistAsc) | count
(== CAT count) | fingerprint (== CAT's). Body: `count × u32 LE` — `LST[i]` = catalog
index of logical position *i*; file length `24 + count×4`. `SortOrder` has four
variants (Title/Artist × Asc/Desc) but only two files exist: **Descending = reverse
iteration of the same file** (`ArtistDescending` reads `ARTISTASC[count-1-i]`, no
second LST). `get_list_filename(SortOrder)` maps every variant to its base file +
direction. Artist order is degenerate until M5 tag parsing fills real `artist`
fields (all `""` ⇒ stable catalog order) — accepted. Future orders share this layout.

### Fingerprint & lifecycle

- **Fingerprint** = FNV-1a u64 over every *eligible* file's `(path, size)` — the DFS
  walks the card hashing incrementally each boot (O(1) RAM, no writes). Skipped files
  are invisible to it (eligible-set semantics: a skipped file changing never triggers a
  rebuild). u64 keeps the silent-miss probability fleet-negligible.
- **Boot**: mount → header validation (`magic` → `version` → `count`-vs-file-length) →
  fingerprint walk → mismatch/missing ⇒ rebuild: one DFS pass streaming unsorted records
  into a fresh CAT **and** appending MCOMP entries (same pass, same fingerprint) → LST
  load-or-build (fingerprint match ⇒ serve; else read CAT sequentially, sort into
  `Vec<Track, CAP=256>`, write). An unchanged card never sorts.
- **Skip policies** (`skipped_count` in the CAT header): chains deeper than 16
  components; reconstructed paths exceeding the ~4KB path budget (bounds the M5 reopen
  buffer). Component names can never overflow (765 B is the format max).
- **Access**: `library::fetch_window(order, start, end)` = open LST → seek to
  `24+i×4` (reverse mapping for Descending) → read `(end-start)` indices → for each,
  seek to `24+idx×580` and read the 580 B CAT record → hydrate `Track` — fixed-offset
  reads riding exfat-slim's slot cache, awaited inside `file_manager_task`.
- **Reopen (M5)**: record → ≤16 MCOMP reads → reconstruct `"/Music/日本語/曲.flac"` →
  open **via a request to `file_manager_task`** (sole FS owner, rule 12) — once per song.
- **`Track` (RAM)**: `{title: String<256>, artist: String<256>, duration_ms: u32,
  cat_index: u32}` (~520 B). The path exists only as the on-card chain.

### Playlist window (externally driven — M4 decision)

The FS reads do **not** live in `Playlist`; the window is fed over the event channel.
`Playlist` = `heapless::Deque<Track, MAX_TRACKS=16>` + absolute logical
`window_start` + relative `current_index` (absolute current = `window_start +
current_index`). `MAX_STRING_LENGTH = 256` drives title/artist/`Label`.

- **Fetch protocol**: `Player` emits `Event::FileManager(Fetch::Playlist { sort_order,
  start_index, end_index })`; `file_manager_task` awaits
  `fetch_window(order, start, end)` and publishes
  `Event::Player(Command::UpdatePlaylist(Box<Playlist>))`.
- **Boot**: `file_manager_task` performs the initial fetch `[0, 16)` itself at startup
  (default `TitleAscending`) before entering its event loop. `Player::new(empty)`
  starts idle — `TrackChanged(None)` + `Toggled(true)` seed the UI; playback begins
  when the first window arrives.
- **Prefetch during playback**: when `current_index + 2 >= window len`, `Player`
  emits `Fetch::Playlist { start: window_end, end: window_end + 16 }` behind a
  `fetch_in_flight` guard (2 tracks of slack ≫ the ~16 × (4 B + 580 B) fetch cost).
  This is the "sorted list opens *while a music file is streaming*" case: with M5
  decode, `file_manager_task` interleaves window fetches and the decoder's chunk
  reads at await points — one owner, no locks.
- **Apply**: `UpdatePlaylist` splices when contiguous (drop the consumed head, append
  the fetched tail, advance `window_start`) and replaces wholesale on a sort-order
  change (keep the current track if present in the new window, else stop).
- **End of library**: `fetch_window` clamps past-the-end ranges; when the window truly
  exhausts, `next()` → `None` ⇒ `Player::stop()` (existing semantics preserved).

### exfat-slim notes (verified against 0.7.0 source)

- Path-based API (`open`, `read_dir`, `create_directory`, `write`, `exists`, `remove_file`,
  `rename`), blocking and async from one source; `OpenOptions` with
  create/truncate/append; `seek(u64)`; full **write** support.
- **Single-owner API shape**: every `File` op takes `&mut FileSystem`
  (`File::read(&mut self, fs, buf)`, `seek`, `write`, `flush`) — a `File` handle alone
  is inert; two tasks cannot each hold working files. Multiple files may be open at
  once under one owner, interleaving at await points. This is why `file_manager_task`
  owns the FS (rules 9/12) and why "streaming a music file while opening the sorted
  list" just works.
- `read_dir` yields `DirectoryEntry { metadata, name }` with `metadata.len()` — the
  fingerprint walk and duration estimator get file sizes from directory iteration,
  no per-file opens needed.
- The `embassy` feature exposes the FS through a crate-internal actor (`fs_actor_task`,
  static `Channel<Req, 8>`, `REPLY_SLOTS = 4`, `FileHandle` client API, BTreeMap
  handle table). **Rejected**: it implies `alloc`, allocates a `Vec` per
  `FileHandle::read` (copy + churn on the 64KB firmware heap), and duplicates the
  actor pattern our event system already provides.
- Names are UTF-16LE on disk, decoded to UTF-8 with a caller-supplied buffer
  (`MAX_NAME_LEN = 765`) — CJK-native, one canonical name per file (no FAT LFN/SFN
  duality; the earlier SFN-path reopen-key design is obsolete).
- `no_std`, `deny(unsafe_code)`, allocator optional; core paths we use are alloc-free.
- The async `BlockDevice` trait (re-exported from `block-device-driver`, native AFIT)
  has `type Align: aligned::Alignment`; the crate's own in-memory example device uses
  `BLOCK_SIZE = 512`, slot cache `N = 4`, `type Align = A4` — `ExFatRamBlockDevice`
  mirrors this (we pick `N = 8` for headroom).
- `mount()` expects the exFAT boot sector at view-sector 0 — adapters must apply the
  MBR partition LBA themselves.
- Known TODOs (accepted risks): reads past `valid_data_length` don't zero-fill (our
  reads are always exact-offset/exact-length); no double-open file locking (single
  ownership in `file_manager_task` makes it moot — re-verify when M5 decoder reads
  interleave in the same task). Mitigations: pinned version, FS layer isolated in
  `library.rs`/`file_manager.rs`, the app-mock behavioral suite, and a documented swap
  path back to embedded-sdmmc if the crate disappoints.

### Simulator image strategy

`app-mock` gunzips the committed `assets/exfat.img.gz` (a small `mkfs.exfat` volume,
regenerated via `tools/make_exfat_image.sh`) into a `Vec`-backed `BlockDevice`, then
populates `assets/music/` **through exfat-slim's own `create_directory`/`write`** —
CJK names created natively, zero hand-crafted on-disk structures. This mirrors the
crate's own examples (17KB gzipped image). Fallback when assets are missing: synthesize
a 2-track population.

---

## Display / Simulator (`app-mock`)

- **Pixel colour:** `Gray8` — grayscale pixels. Chosen to match the target monochrome LCD display.
- **Placeholder resolution:** 240×240. Change `DISPLAY_WIDTH` / `DISPLAY_HEIGHT` in
  `ui/display_config.rs` once the real display is chosen.
- **Scale:** `PIXEL_SCALE = 3` (with `PIXEL_SPACING = 0`) — zooms the desktop window to a
  comfortable size.
- **Font:** `FONT_6X10` is ASCII-only — CJK titles are stored/sorted/reopened correctly
  but won't *render* until a CJK-capable font task happens (post-M4 UI work).
- **Keyboard shortcuts in simulator:**
  - `Space` — toggle play/pause
  - `M` — toggle Settings / Main screen
  - `N` — next track
  - `P` — previous track
  - `1`–`9` — seek to 10%–90%; `0` — seek to 0%
  - `Escape / close` — quit
- **SDL2 required on host:** install via your distro (e.g. `libsdl2-dev`) — the
  `embedded-graphics-simulator` window depends on it.
- **Note:** `app-mock` runs the real Embassy executor on the host
  (`embassy-executor` with `platform-std`, `embassy-time` with `std`), so async task code
  written here transfers directly to the firmware.
- Run with: `cargo run -p app-mock` (from workspace root — uses host target).

---

## GitLab CI/CD Pipeline (PLANNED — no `.gitlab-ci.yml` yet)

Four stages planned in `.gitlab-ci.yml`:

| Stage | Runner | Trigger |
|---|---|---|
| `check` (clippy) | Shared | Every push and MR |
| `test` (host unit tests) | Shared | Every push and MR |
| `build` (cross-compile firmware) | Shared | Every push and MR |
| `flash` (probe-rs to hardware) | Self-hosted (tagged `hardware,stm32`) | `develop` and `main` only |

Build command for firmware in CI always passes the target explicitly:
```bash
cargo build -p app-firmware --release --target thumbv7em-none-eabihf
```

Host tests never cross-compile:
```bash
cargo test -p app-core -p app-mock
```

---

## Event Architecture

All inter-module communication uses a unified `Event` enum:

```rust
pub enum Event {
    ButtonPress(Button),            // Input layer (SDL2 / GPIO)
    Player(Command),                // Commands TO the Player (Toggle, Seek, NextTrack,
                                    //   UpdatePlaylist, …)
    Playback(State),                // State changes FROM the Player (TrackChanged, …)
    Ui(Screen),                     // Screen transitions, refresh requests
    FileManager(Fetch),             // Read requests TO the file manager (Fetch::Playlist
                                    //   { sort_order, start_index, end_index }, …)
}
```

The `Player(Command)` / `Playback(State)` split keeps direction explicit: commands flow
into the player handler, state changes flow out; `FileManager(Fetch)` is the same idea
for card reads. The `EventHandler` trait unifies this:

```rust
pub trait EventHandler {
    fn event_queue(&mut self) -> &mut EventQueue;
    fn handle_event(&mut self, event: &Event) -> EventQueue;
    async fn handle_event_async(&mut self, event: &Event) -> EventQueue {  // (M4)
        self.handle_event(event)                                        // default: sync path
    }
    fn push_events(&mut self) -> EventQueue;  // drains the internal queue
    fn add_event(&mut self, event: Event);
}
```

`Task::service_event` awaits `handle_event_async` (M4). Handlers with no async needs
(Player, screens) inherit the default; `FileManager` overrides it for its FS reads
(rule 11).

### Handler boundaries

Handlers implement `EventHandler`; the "actor model" is the communication pattern
(own state + message passing via event queues) they follow.

| Handler | Owned by | Receives | Emits |
|---|---|---|---|
| **Player** | `player_task` | `Event::Player(Command::*)` incl. `UpdatePlaylist`, 100ms `Tick` | `Playback(State::*)`: `TrackChanged`, `Toggled`, `ProgressUpdated`, `Stopped`; `FileManager(Fetch::Playlist)` prefetch near the window edge |
| **FileManager** | `file_manager_task` | `Event::FileManager(Fetch::*)` | `Player(Command::UpdatePlaylist)` — **sole owner of the exFAT `FileSystem`** |
| **Decoder** | `decode_task` (M5) | `cat_index` / seek commands from Player; streams its file **via `file_manager_task`** (mechanism TBD at M5) | `DecodeError`, buffered PCM |
| **Active Screen** | `ui_task` | `ButtonPress`, `Playback(State::*)` | `Player` commands, `Ui(Refresh)`, `Ui(Change)` |
| **ScreenManager** | `ui_task` | `Ui(Change)` | Routes events to active screen |

### Task bridging (app-mock, and later app-firmware)

- A static `PubSubChannel<CriticalSectionRawMutex, Event, 64, 3, 3>` (`EventChannel` in
  `app-mock/src/tasks.rs`; counts 2→3 in M4 when `file_manager_task` joins) is shared
  by all tasks. Each task wraps its handler in the generic `Task<T: EventHandler>`
  struct, which owns a publisher + subscriber endpoint, awaits
  `handler.handle_event_async()`, then publishes the resulting events.
- **player_task** selects between `subscriber.next_message()` and a 100ms `Ticker`.
  It services `Event::Player(_)` commands and injects `Command::Tick(100)` on timeout.
  It publishes everything its handler emits — including `FileManager(Fetch::Playlist)`
  prefetch requests.
- **ui_task** selects between `subscriber.next_message()` and a 33ms timeout. It
  services `Playback(_)` state changes (which schedule `Screen::Refresh`) and
  `Ui(Screen::Change)`, and polls SDL2 for keyboard input on timeout.
- **file_manager_task** (M4, the FS actor): at startup it constructs
  `FileSystem::new(ExFatRamBlockDevice)` → `mount()` → `Library::open_or_build()`
  (boot scan) → boot fetch `[0, 16)` → publishes `UpdatePlaylist`; then it loops on
  `subscriber.next_message()` servicing `Event::FileManager(_)`. It never touches the
  `Event::Player(_)` / `Playback(_)` traffic.
- **Spawn order matters (M4):** `ui_task` → `player_task` → `file_manager_task`.
  Embassy pubsub drops messages published before a subscriber joins, and `Task::new`
  subscribes synchronously at task start, so earlier-spawned tasks are guaranteed
  subscribed before `file_manager_task`'s first publish (its boot scan awaits, which
  yields to them first). Document this in `main.rs`.
- **Event echo control:** each task filters which channel messages it services, so
  events published by a task are not re-processed by it (no infinite loops).

### Initialization

`Player::new(playlist)` emits `TrackChanged`, `Toggled(true)`, and `ProgressUpdated`
events into its internal queue at construction, bootstrapping the first UI draw
without a special-case initial-sync path. In M4 the constructor is called with an
**empty** `Playlist`: the emitted `TrackChanged(None)` seeds the UI while
`file_manager_task` mounts, scans, fetches the first window `[0, 16)`, and publishes
`UpdatePlaylist` — playback of track 0 begins on arrival. The ui_task performs an
initial `refresh_screen` before its event loop starts. (Until M4 lands,
`mock_player::new()` still seeds a hardcoded 2-track playlist.)

---

## Audio Pipeline (planned target state)

FLAC/MP3 decoding will be handled by a `Decoder` handler in `app-core` (not yet implemented — Milestone 5):

```
SD card bytes → AudioFileReader → Decoder (Symphonia) → AudioOutput → DAC
```

`Decoder` is platform-agnostic. It reopens a track via `Track.cat_index` → `.CAT`
chain → `MCOMP.CAT` entries → reconstructed path, then streams the file **through
`file_manager_task`** (sole FS owner, rule 12 — the decoder never owns the
`FileSystem` itself). The request mechanism is decided at M5: leading candidate is a
zero-copy lending channel (decoder lends `&'static mut [u8]` buffers that the file
manager fills via `File::read`); the fallback is event-based chunk requests. The
`AudioFileReader`/`AudioOutput` ports in `hal.rs` remain the decoder's I/O boundary —
how the reader is backed (a file_manager-backed implementation vs a mock) is an M5
detail. On the host, both sides are mocked. On the STM32, the exFAT block device and
`Ak4377Output` provide real I/O. Duration/tag parsing at M5 also fills real
`title`/`artist`/`duration_ms` into the catalog records (replacing filename-derived
values and size-based estimates).

The decoder will run in its own async task, filling a bounded PCM buffer. `Player` will
consume samples from this buffer each `tick()`, advancing `elapsed_ms` based on actual
sample consumption rather than a simulated timer.

**Today**, `Player::tick(delta_ms)` simply advances `elapsed_ms` by a simulated time delta
(the desktop simulator's `player_task` injects a 100ms `Command::Tick` via an Embassy
`Ticker`), so progress advances even without decoding.

---

## Milestones

- [x] **Milestone 1** Retained-mode GUI framework (`app-core` + `app-mock`) — widget structs (`Label`, `ProgressBar`, `PlayButton`, `List`), screen composition via `Default`, `app-mock` update/sync/draw loop with keyboard navigation
- [x] **Milestone 2** Player state machine + playlist logic (`app-core`, host-tested) — actor model with event-driven state changes, auto-advance, next/prev
- [x] **Milestone 3** Transport controls + progress bar UI (event-driven, auto-advance) — `EventHandler` trait + `PubSubChannel` task bridging running on the real Embassy executor in `app-mock`
- [ ] **Milestone 4** SD library + playlist database (exFAT, locked design — see
  **SD Library & On-Card Catalog**; in progress on branch `sorted-list-implementation`).
  Landed already: `Deque<Track,16>` playlist, `MAX_STRING_LENGTH = 256`,
  `Fetch::Playlist`/`UpdatePlaylist` event plumbing, `FileManager` handler skeleton,
  4-variant `SortOrder`. Remaining work (stages, in order):
  - **Stage 0 — unbreak build + deps:** `get_list_filename -> &'static str` (fixes
    E0573 at `file_manager.rs:48`); workspace gains `exfat-slim 0.7`
    (`default-features = false`) and `miniz_oxide` (app-mock); drop the unused
    `embedded-sdmmc` pin
  - **Stage 1 — async trait:** `EventHandler::handle_event_async` (default = sync);
    `Task::service_event` awaits it; `EventChannel` counts 2→3 pub / 3 sub
  - **Stage 2 — track/playlist/player:** `Track.index: usize` → `cat_index: u32`;
    `Playlist.window_start` + splice methods; `Player` window logic (empty at boot,
    prefetch emit at `current_index + 2 >= len` behind a `fetch_in_flight` guard,
    splice-or-replace on `UpdatePlaylist`); update `mock_player.rs` +
    player/playlist tests
  - **Stage 3 — `app-core/src/library.rs`:** pure codecs (24B headers, 580B CAT
    record, 765B MCOMP entry, FNV-1a, stem derivation, extension filter, duration
    estimator, SortOrder comparator, chain→path) + async fns taking
    `&mut FileSystem` (boot scan: iterative DFS, streaming CAT/MCOMP rebuild, LST
    load-or-build; `fetch_window(order, start, end)`)
  - **Stage 4 — `FileManager` + task:** `FileManager<B: BlockDevice<512>, N>` owns
    `FileSystem<B, 512, 8>`; `handle_event_async` services `Fetch::Playlist`;
    `file_manager_task`: mount → `Library::open_or_build` → boot fetch `[0,16)` →
    loop; spawn order ui → player → file_manager (see **Event Architecture**)
  - **Stage 5 — sim plumbing:** `tools/make_exfat_image.sh` +
    `assets/exfat.img.gz`; `sd_image.rs` `ExFatRamBlockDevice` (miniz_oxide gunzip,
    async `BlockDevice<512>`, `Align = A4`, volume at view-sector 0) +
    `populate_from_dir` (CJK samples) + synthetic 2-track fallback
  - **Stage 6 — tests:** pure-logic in `app-core/src/library/tests.rs` (codecs, FNV,
    derivations, comparator, chain→path); behavioral suite in `app-mock/src/tests/`
    (scan→CAT/MCOMP, LST asc/desc, fingerprint change/stale, interning dedup, window
    slides, player e2e with prefetch firing during playback, reopen-by-CJK-path)
- [ ] **Milestone 5** FLAC decode via Symphonia (desktop end-to-end) — `app-core/src/decoder.rs`, real duration/tag parsing from FLAC STREAMINFO / MP3 headers written back into catalog records, reopen-by-chain, **music-file streaming through `file_manager_task`** (sole FS owner; request mechanism TBD — see **Audio Pipeline**), host end-to-end test
- [ ] **Milestone 6** Audio mocks — `MockFileReader`/`MockAudioOutput` in `app-core/src/hal/mock.rs` (traits exist in `hal.rs`)
- [ ] **Milestone 7** Hardware bringup (STM32 boot, LED blink, RTT logs)
- [ ] **Milestone 8** AK4377 I2C init + SAI sine wave
- [ ] **Milestone 9** SD card reads + raw PCM playback — exfat-slim `BlockDevice` adapter
  over embassy-stm32 SDMMC **plus MBR type-0x07 partition-offset translation**
- [ ] **Milestone 10** Combine file read and play song on AK4377


## Running Common Tasks

```bash
# Host unit tests (no hardware needed; includes app-mock's behavioral suite)
cargo test -p app-core -p app-mock

# Run the desktop UI simulator
cargo run -p app-mock

# Regenerate the committed empty exFAT image (requires mkfs.exfat from exfatprogs)
tools/make_exfat_image.sh

# Build firmware (cross-compile)
cargo build -p app-firmware --release --target thumbv7em-none-eabihf

# Flash and stream RTT logs (from app-firmware/ directory)
cd app-firmware && cargo run --release

# Lint entire workspace
cargo clippy --workspace -- -D warnings

# Format entire workspace
cargo fmt --all

# Check firmware binary size
cargo size -p app-firmware --release -- -A
```