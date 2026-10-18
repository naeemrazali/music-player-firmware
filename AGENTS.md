# agents.md — Music Player Firmware

## Project Summary

Embedded Rust portable audio player built on the **STM32H7A3VI** microcontroller using the
**Embassy** async runtime. Audio files are read from an **exFAT** SD card, decoded by
**Symphonia** (MP3 and FLAC), and streamed as PCM over **SAI** to an **AK4377**
DAC/headphone amplifier chip which handles digital-to-analogue conversion and drives
the headphones.

The filesystem layer is **exfat-slim 0.7.0** (pinned, `no_std`), used directly inside
`app-core` as the shared FS for the desktop simulator and the firmware — host and target
differ only in their `BlockDevice` implementation. Music metadata lives in three
on-card database files (`MUSICLIB.CAT` catalog + `MCOMP.CAT` interned-component table +
`TITLEASC.LST` sorted view), so RAM usage is O(1) in library size and sorting never
repeats for an unchanged card. See **SD Library & On-Card Catalog** for the full design.

The UI and player logic communicate via an **actor-model event system** using
`heapless::Vec<Event, 8>` internal queues, formalised by the `EventHandler` trait.
A static Embassy `PubSubChannel` bridges these queues between async tasks — this is
no longer just planned: the desktop simulator runs the real Embassy executor
(`platform-std`) with `ui_task` and `player_task` spawned from `#[embassy_executor::main]`,
so the task/channel design is validated on the host before firmware bringup. The same
design carries over to the STM32 firmware tasks.

A desktop **mock** crate mirrors the embedded display using `embedded-graphics-simulator`
so UI code can be developed and tested without hardware.

### Current status

- **Done:** retained-mode GUI framework, `Player` + `Playlist` state machines (actor-model
  pattern, `EventHandler` trait, event queues), transport controls + progress bar UI,
  `app-mock` simulator running the real Embassy executor with a static `PubSubChannel`
  bridging `ui_task` / `player_task`. `app-hal` dissolved — its traits live in
  `app-core/src/hal.rs` (1ff5737). `Track` moved from `&'static str` to `heapless::String`
  fields, then to `{title, artist, duration_ms, file_path}` with `MAX_STRING_LENGTH = 64`
  (6547842, 3035ad0).
- **In flight — build currently broken:** `Track.file_index` was replaced by `file_path`
  (3035ad0) but the ripple is unfinished — `app-mock/src/mocks/mock_player.rs` (2 compile
  errors) and the `player/tests.rs` / `playlist/tests.rs` helpers still construct
  `file_index`. The M4 rework below replaces all three files outright, clearing the
  breakage and the 3 known `Playlist` clippy lints (`new_without_default`,
  `result_unit_err`, `should_implement_trait`) with the playlist rewrite.
- **Decided for M4 (locked design, not yet implemented):** exFAT-only storage via
  exfat-slim 0.7.0; three on-card database files with a FNV-1a generation fingerprint
  scanned every boot; `Track` drops `file_path` in favour of a `cat_index` pointing at
  its `.CAT` record (string interning: the record's 16-deep `u32` chain references
  directory/filename strings in `MCOMP.CAT`); `Playlist` becomes a `heapless::Deque<Track, 16>`
  sliding window over the sorted view; the simulator mounts a committed gzipped exFAT
  image in RAM and populates it through exfat-slim's own write path (CJK samples
  included). See **SD Library & On-Card Catalog** and **Milestones**.
- **Not started:** Symphonia decoding (`app-core/src/decoder.rs`), `memory.x`, CI pipeline,
  and all hardware bringup (the `app-firmware` crate is still a hello-world placeholder
  with no dependencies and does not yet build for the embedded target).

---

## Hardware

| Component | Part | Notes |
|---|---|---|
| Microcontroller | STM32H7A3VI (LQFP100) | Cortex-M7 @ 280MHz, 2MB Flash, 1.4MB SRAM |
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
├── app-core/                       # Pure business logic — no hardware, no_std-ready
│   └── src/
│       ├── lib.rs                  # Module declarations
│       ├── event.rs                # Button, Command, State, Screen, Event enums +
│       │                           #   EventHandler trait, EventQueue type
│       ├── player.rs               # Player handler — implements EventHandler, owns state
│       ├── player/
│       │   └── tests.rs            # Player pure-logic tests (submodule pattern)
│       ├── playlist.rs             # (M4 rewrite) Deque<Track,16> window over the Library
│       ├── playlist/
│       │   └── tests.rs           # Playlist pure-logic tests
│       ├── track.rs                # (M4 target) {title: String<256>, artist: String<256>,
│       │                           #   duration_ms: u32, cat_index: u32} — no path field
│       ├── hal.rs                  # AudioFileReader, AudioOutput traits + error enums
│       │                           #   (M4 decision: NO Filesystem trait — exfat-slim IS
│       │                           #    the shared FS layer, used directly in library.rs)
│       ├── library.rs              # (M4, planned) SortOrder, FNV-1a fingerprint DFS,
│       │                           #   catalog/MCOMP rebuild, LST load-or-build,
│       │                           #   two-level entry(logical_i)
│       ├── library/
│       │   └── tests.rs            # Pure-logic tests: record/table/header codecs, FNV
│       │                           #   vectors, title derivation, extension filter,
│       │                           #   duration estimator, SortOrder comparator
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
        │                           #   spawns ui_task + player_task
        ├── tasks.rs                # Task<T: EventHandler> struct (owns handler + pub/sub
        │                           #   endpoints), EventChannel type aliases
        ├── tasks/
        │   ├── player_task.rs      # Player task — mounts image+Library+Playlist+Player,
        │   │                       #   services Player commands + 100ms tick
        │   └── ui_task.rs          # UI task — keyboard input, screen refresh loop
        ├── mocks.rs                # mock_display, mock_window, mock_player constructors
        ├── mocks/
        │   ├── mock_display.rs
        │   ├── mock_player.rs      # (M4 rewrite) image→populate→Library→Playlist→Player
        │   └── sd_image.rs         # (M4, planned) ExFatRamBlockDevice (gunzip→Vec, impl
        │                           #   exfat-slim BlockDevice<512>) + populate_from_dir()
        │                           #   via exfat-slim create_directory/write
        └── tests/                   # (M4, planned) behavioral suite, #[cfg(test)] — the
                                    #   image tooling is simulator-only, so all tests that
                                    #   need a mounted volume live here: scan→CAT, LST
                                    #   ordering, fingerprint change/stale, interning
                                    #   dedup, chain→path reconstruction, window slides,
                                    #   player e2e, reopen-by-CJK-path
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

9. **exfat-slim is the shared FS layer, used directly in `app-core`; `BlockDevice`
   adapters live at the edges.** `library.rs` calls exfat-slim's path-based API
   (`open`, `read_dir`, `create_directory`, `write`, `seek`). `app-mock` supplies
   `ExFatRamBlockDevice` (gunzipped committed image); `app-firmware` supplies the
   embassy-stm32 SDMMC adapter **plus MBR partition-offset translation** (exfat-slim
   mounts at view-sector 0; real SDXC cards carry an MBR with a type-0x07 exFAT
   partition whose LBA the adapter must add to every sector address — M9).
   `hal.rs` keeps `AudioFileReader`/`AudioOutput` as the M5 decoder's ports.
   (Supersedes the earlier embedded-sdmmc/FAT32 plan and the `Filesystem` trait.)

10. **The on-card database files are private formats; the fingerprint is the glue.**
    Any byte layout change to `MUSICLIB.CAT` / `MCOMP.CAT` / `TITLEASC.LST` bumps its
    `version` field (old files auto-rebuild). All three files of one generation share
    one fingerprint; never mix generations.

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
  (see *SD Library & On-Card Catalog* → *exfat-slim notes*). `default = []`
  (no logging, no alloc, no embassy); features exist: `alloc`, `defmt`, `embassy`
  (fs actor task — candidate for M5+), `log`.
- `miniz_oxide` in `app-mock` only — gunzips `assets/exfat.img.gz` at startup.
- The old `embedded-sdmmc` workspace pin is removed (superseded by the exFAT decision).

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
| AXISRAM | `Library` state + `Deque<Track,16>` window (16 × ~520B) | ~9KB |
| AXISRAM | LST build sort buffer `Vec<Track, CAP=256>` (~520B/entry) | ~148KB, boot-transient only |
| AXISRAM | DFS walk buffers (765B name buf, ~4KB path prefix) | ~5KB, boot-transient only |

The static `PubSubChannel<Event, 64>` also shrinks ~32KB on firmware once `Track`
drops its path field (Event enum size falls ~512B/slot).

---

## SD Library & On-Card Catalog (M4 design — locked)

RAM usage is O(1) in library size: the sorted lists live **on the SD card**; the
in-memory `Library` is a ~40-byte accessor. Three private files in the volume root,
all rebuilt together (one fingerprint, one generation):

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

### `/TITLEASC.LST` — sorted view (u32 catalog indices)

Header **24 B**: `magic "MLST"` | version | order (u32 LE, `SortOrder` discriminant:
0 = NameAsc, 1 = NameDesc) | count (== CAT count) | fingerprint (== CAT's).
Body: `count × u32 LE` — `LST[i]` = catalog index of logical position *i*; file
length `24 + count×4`. `SortOrder::NameDesc` = **reverse iteration of the same file**
(no second LST). Future orders (M5+: `ARTISTASC.LST` etc.) share this layout.

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
- **Access**: `Library::entry(logical_i)` = read `LST[i]` (4 B at `24+i×4`) → read
  `CAT[idx]` (580 B at `24+idx×580`) → hydrate `Track` — two fixed-offset reads, both
  riding exfat-slim's slot cache.
- **Reopen (M5)**: record → ≤16 MCOMP reads → reconstruct `"/Music/日本語/曲.flac"` →
  `fs.open(path)` — once per song.
- **`Track` (RAM)**: `{title: String<256>, artist: String<256>, duration_ms: u32,
  cat_index: u32}` (~520 B). The path exists only as the on-card chain.

### Playlist window

`Playlist` owns `Library` + `heapless::Deque<Track, 16>` + absolute logical
`window_start`/`current_index`. `next()`/`prev()` preserve the current semantics
(clamp at start; `None` at end ⇒ Player stops) and slide the deque at the edges —
one pop + one two-level read per step. `MAX_STRING_LENGTH = 256` drives
title/artist/`Label`.

### exfat-slim notes (verified against 0.7.0 source)

- Path-based API (`open`, `read_dir`, `create_directory`, `write`, `exists`, `remove_file`,
  `rename`), blocking and async from one source; `OpenOptions` with
  create/truncate/append; `seek(u64)`; full **write** support.
- Names are UTF-16LE on disk, decoded to UTF-8 with a caller-supplied buffer
  (`MAX_NAME_LEN = 765`) — CJK-native, one canonical name per file (no FAT LFN/SFN
  duality; the earlier SFN-path reopen-key design is obsolete).
- `no_std`, `deny(unsafe_code)`, allocator optional; core paths we use are alloc-free.
- `mount()` expects the exFAT boot sector at view-sector 0 — adapters must apply the
  MBR partition LBA themselves.
- Known TODOs (accepted risks): reads past `valid_data_length` don't zero-fill (our
  reads are always exact-offset/exact-length); no double-open file locking (single-task
  FS ownership in M4 makes it moot). Mitigations: pinned version, FS layer isolated in
  `library.rs`, the app-mock behavioral suite, and a documented swap path back to
  embedded-sdmmc if the crate disappoints.

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
    Player(Command),                // Commands TO the Player (Toggle, Seek, NextTrack, …)
    Playback(State),                // State changes FROM the Player (TrackChanged, …)
    Ui(Screen),                     // Screen transitions, refresh requests
}
```

The `Player(Command)` / `Playback(State)` split keeps direction explicit: commands flow
into the player handler, state changes flow out. The `EventHandler` trait unifies this:

```rust
pub trait EventHandler {
    fn event_queue(&mut self) -> &mut EventQueue;
    fn handle_event(&mut self, event: &Event) -> EventQueue;
    fn push_events(&mut self) -> EventQueue;  // drains the internal queue
    fn add_event(&mut self, event: Event);
}
```

### Handler boundaries

Handlers implement `EventHandler`; the "actor model" is the communication pattern
(own state + message passing via event queues) they follow.

| Handler | Owned by | Receives | Emits |
|---|---|---|---|
| **Player** | `player_task` | `Event::Player(Command::*)`, 100ms `Tick` | `Playback(State::*)`: `TrackChanged`, `Toggled`, `ProgressUpdated`, `Stopped` |
| **Decoder** | `decode_task` (planned) | `cat_index` / seek commands from Player; reopens via chain→MCOMP→path | `DecodeError`, buffered PCM |
| **Active Screen** | `ui_task` | `ButtonPress`, `Playback(State::*)` | `Player` commands, `Ui(Refresh)`, `Ui(Change)` |
| **ScreenManager** | `ui_task` | `Ui(Change)` | Routes events to active screen |

### Task bridging (app-mock, and later app-firmware)

- A static `PubSubChannel<CriticalSectionRawMutex, Event, 64, 2, 2>` (`EventChannel` in
  `app-mock/src/tasks.rs`) is shared by all tasks. Each task wraps its handler in the
  generic `Task<T: EventHandler>` struct, which owns a publisher + subscriber endpoint
  and calls `handler.handle_event()`, then publishes the resulting events.
- **player_task** selects between `subscriber.next_message()` and a 100ms `Ticker`.
  It services `Event::Player(_)` commands and injects `Command::Tick(100)` on timeout.
- **ui_task** selects between `subscriber.next_message()` and a 33ms timeout. It
  services `Playback(_)` state changes (which schedule `Screen::Refresh`) and
  `Ui(Screen::Change)`, and polls SDL2 for keyboard input on timeout.
- **Event echo control:** each task filters which channel messages it services, so
  events published by a task are not re-processed by it (no infinite loops).

### Initialization

`Player::new(playlist)` seeds the initial state: it emits `TrackChanged`,
`Toggled(true)`, and `ProgressUpdated` events into its internal queue at construction,
bootstrapping the first UI draw without a special-case initial-sync path. In the
simulator, `mock_player::new()` builds the RAM exFAT image (or the synthetic fallback),
mounts the `Library`, opens the `Playlist`, constructs the `Player`, and the ui_task
performs an initial `refresh_screen` before its event loop starts.

---

## Audio Pipeline (planned target state)

FLAC/MP3 decoding will be handled by a `Decoder` handler in `app-core` (not yet implemented — Milestone 5):

```
SD card bytes → AudioFileReader → Decoder (Symphonia) → AudioOutput → DAC
```

`Decoder` is platform-agnostic. It reopens a track via `Track.cat_index` → `.CAT`
chain → `MCOMP.CAT` entries → reconstructed path → exfat-slim `open()`, then reads
through the `AudioFileReader` port and writes PCM through `AudioOutput`. On the host,
both sides are mocked. On the STM32, the exFAT block device and `Ak4377Output` provide
real I/O. Duration/tag parsing at M5 also fills real `title`/`artist`/`duration_ms`
into the catalog records (replacing filename-derived values and size-based estimates).

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
  **SD Library & On-Card Catalog**):
  - `track.rs`: `MAX_STRING_LENGTH` 64→256; `Track` → `{title, artist, duration_ms,
    cat_index}` (`file_path` removed; fixes the current broken build and the 3 known
    `Playlist` clippy lints)
  - `app-core/src/library.rs`: `SortOrder`, FNV-1a fingerprint DFS, streaming CAT+MCOMP
    rebuild, lazy fingerprint-bound LST build, two-level `entry(logical_i)`
  - `app-core/src/playlist.rs` rewrite: `Deque<Track, 16>` window (probe non-`Copy`
    support first; `Vec`+rotate fallback)
  - `app-mock`: `sd_image.rs` (gunzip + `ExFatRamBlockDevice` + `populate_from_dir`
    via exfat-slim writes), committed `assets/exfat.img.gz` +
    `tools/make_exfat_image.sh`, CJK sample assets, `player_task` wiring
  - Tests: pure-logic in `app-core` (codecs, FNV, derivations, comparator);
    behavioral suite in `app-mock` (scan→CAT, LST, fingerprint change/stale, interning
    dedup, chain→path reconstruction, window slides, player e2e, reopen-by-CJK-path)
- [ ] **Milestone 5** FLAC decode via Symphonia (desktop end-to-end) — `app-core/src/decoder.rs`, real duration/tag parsing from FLAC STREAMINFO / MP3 headers written back into catalog records, reopen-by-chain, host end-to-end test
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