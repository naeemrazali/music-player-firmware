# agents.md — Music Player Firmware

## Project Summary

Embedded Rust portable audio player built on the **STM32H7A3VI** microcontroller using the
**Embassy** async runtime. Audio files are read from an SD card, decoded by **Symphonia**
(MP3 and FLAC), and streamed as PCM over **SAI** to an **AK4377** DAC/headphone amplifier
chip which handles digital-to-analogue conversion and drives the headphones.

The UI and player logic communicate via an **actor-model event system** using
`heapless::Vec<Event, 8>` internal queues, formalised by the `EventHandler` trait.
A static Embassy `PubSubChannel` bridges these queues between async tasks — this is
no longer just planned: the desktop simulator runs the real Embassy executor
(`arch-std`) with `ui_task` and `player_task` spawned from `#[embassy_executor::main]`,
so the task/channel design is validated on the host before firmware bringup. The same
design carries over to the STM32 firmware tasks.

A desktop **mock** crate mirrors the embedded display using `embedded-graphics-simulator`
so UI code can be developed and tested without hardware.

### Current status

- **Done:** retained-mode GUI framework, `Player` + `Playlist` state machines (actor-model
  pattern, `EventHandler` trait, event queues), transport controls + progress bar UI,
  `app-hal` trait definitions, `app-mock` simulator running the real Embassy executor with
  a static `PubSubChannel` bridging `ui_task` / `player_task`. 15 host unit tests pass.
  Recent changes: `Task`'s actor field/accessor renamed to `handler` (cee3100); `Track`
  migrated from `&'static str` to `heapless::String` fields (72cd7a7) so tracks can come
  from runtime discovery instead of hardcoded literals.
- **In flight — build currently broken:** the `Track` string migration is committed but
  its ripple is unfinished (8 errors in `app-core`). To finish it: drop `Copy` from
  `State` and `Event` in `event.rs` (`Button`/`Command`/`Screen` keep it); `.copied()` →
  `.cloned()` in `player.rs` (`current_track`, `next_track`, `prev_track`);
  `set_text(&track.title)` / `set_text(&track.artist)` in `main_screen.rs`; `.into()` for
  string literals in `playlist/tests.rs`, `player/tests.rs`, `mock_player.rs`.
- **Known broken (lints):** once compiling again, `cargo clippy --workspace -- -D warnings`
  still fails with 3 `Playlist` lints (`new_without_default`, `result_unit_err` on `add`,
  `should_implement_trait` on `next`) — currently masked by the compile errors. The 4th
  lint (derivable `Default` on `Track`) was resolved by the String migration. Fix before
  enabling clippy in CI.
- **Decided for M4 (not yet implemented):** `app-hal` dissolves — all platform-agnostic
  traits (`AudioFileReader`, `AudioOutput`, new `Filesystem`) move to
  `app-core/src/hal.rs` ("ports in the core, adapters at the edges": `app-core` is their
  only consumer, so the pass-through crate adds no value). Mocks: `MockFilesystem`
  (in-memory, pure `no_std`) in `app-core/src/hal/mock.rs`; `HostFilesystem` (`std::fs`)
  in `app-mock/src/mocks/host_fs.rs`. The volume scanner lives in `app-core/src/library.rs`.
  See Milestones.
- **Not started:** Symphonia decoding (`app-core/src/decoder.rs`), `memory.x`, CI pipeline,
  and all hardware bringup (the `app-firmware` crate is still a hello-world placeholder
  with no dependencies and does not yet build for the embedded target).

---

## Hardware

| Component | Part | Notes |
|---|---|---|
| Microcontroller | STM32H7A3VI (LQFP100) | Cortex-M7 @ 280MHz, 2MB Flash, 1.4MB SRAM |
| DAC / Amp | AK4377AECB | Controlled via I2C; audio streamed via SAI (I2S-compatible) |
| Storage | SD card | Connected via SDMMC peripheral (4-bit mode, 3.3V) |
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
├── rust-toolchain.toml             # Pinned nightly toolchain (nightly-2025-07-01)
├── memory.x                        # (planned, not yet created) linker memory regions
├── .cargo/
│   └── config.toml                 # `[target.thumbv7em-none-eabihf]` runner + rustflags only
│                                   # NO [build] target here — see app-firmware below
│
├── app-hal/                        # HAL traits ONLY — (DECIDED: dissolves into
│   └── src/                        #   app-core/src/hal.rs in M4; do not add
│       └── lib.rs                  # AudioFileReader, AudioOutput traits + error enums
│                                   # (a `mock` module is sketched out in comments,
│                                   #  gated behind a `std` feature, but not active)
│
├── app-core/                       # Pure business logic — no hardware dependencies
│   └── src/
│       ├── lib.rs                  # Module declarations
│       ├── event.rs                # Button, Command, State, Screen, Event enums +
│       │                           #   EventHandler trait, EventQueue type
│       ├── player.rs               # Player handler — implements EventHandler, owns state
│       ├── player/
│       │   └── tests.rs            # Player unit tests (submodule pattern)
│       ├── playlist.rs             # Fixed-capacity (32) track list
│       ├── playlist/
│       │   └── tests.rs           # Playlist unit tests (submodule pattern)
│       ├── track.rs                # Track metadata (Clone, heapless::String fields)
│       ├── hal.rs                  # (M4, planned) platform-agnostic ports:
│       │                           #   Filesystem, AudioFileReader, AudioOutput,
│       │                           #   DirEntry, FileError, OutputError
│       ├── hal/
│       │   └── mock.rs             # (M4, planned) MockFilesystem — in-memory tree,
│       │                           #   pure no_std, no feature gate
│       ├── library.rs              # (M4, planned) volume scanner + track database:
│       │                           #   scan(fs, root) -> Playlist from real file
│       │                           #   discovery instead of hardcoded Tracks
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
│       └── main.rs                 # (placeholder hello-world — no dependencies yet)
│
└── app-mock/                       # Desktop simulator — std, runs real Embassy executor
    ├── Cargo.toml                  # edition = "2024"
    └── src/
        ├── main.rs                 # #[embassy_executor::main], static PubSubChannel,
        │                           #   spawns ui_task + player_task
        ├── tasks.rs                # Task<T: EventHandler> struct (owns handler + pub/sub
        │                           #   endpoints), EventChannel type aliases
        ├── tasks/
        │   ├── player_task.rs      # Player task — services Player commands + 100ms tick
        │   └── ui_task.rs          # UI task — keyboard input, screen refresh loop
        ├── mocks.rs                # mock_display, mock_window, mock_player constructors
        └── mocks/
            └── host_fs.rs          # (M4, planned) HostFilesystem — std::fs impl for
                                    #   the simulator's Filesystem port
```

---

## Architecture Rules

1. **Hardware code lives only in `app-firmware/src/drivers/` and `app-firmware/src/tasks/`.**
   No Embassy types, no STM32 peripheral types anywhere in `app-core`.

2. **`app-core` must compile with `cargo test` on the host** (std enabled).
   Never add `embassy-*`, `cortex-m`, or any `no_std`-only dep to `app-core` without
   also gating it behind `#[cfg(not(test))]`.

3. **All drawing code uses `DrawTarget<Color = Gray8>`** — never reference
   `SimulatorDisplay` directly outside of `app-mock/src/mocks/mock_display.rs`
   (where the `MockSimulatorDisplay` alias is defined). This keeps UI code
   portable to the real display driver with zero changes.

4. **PCM DMA buffers go in AXISRAM** via `#[link_section = ".axisram"]`. Do not place
   DMA buffers in DTCM — the DMA controller cannot access it.

5. **Use `defmt` for all firmware logging** — never `println!` or the `log` crate in
   `app-firmware`. Use `#[derive(defmt::Format)]` instead of manual `Debug` impls.

6. **Symphonia requires a global allocator** — `embedded-alloc` is used. Budget ~64KB
   for the heap. FLAC and MP3 are safe; AAC/Opus are marginal at this heap size.

7. **Actor model for UI/Player communication.** `Player` owns its state
   exclusively. Other modules send commands via `Event::Player(...)` and
   receive state changes via events emitted into a bounded `heapless::Vec`.
   No module accesses `Player` fields directly.

8. **Unit tests live in sibling submodules, never inline.** Place tests in
   `src/<module>/tests.rs` and declare them with `#[cfg(test)] mod tests;`
   inside `src/<module>.rs`. This keeps source files readable while retaining
   `use super::*` access to private items.

9. **Ports live in `app-core/src/hal.rs`; adapters live at the edges.** Platform-agnostic
   traits (`Filesystem`, `AudioFileReader`, `AudioOutput`) and their supporting types
   (`DirEntry`, `FileError`, `OutputError`) are defined in the core, next to their only
   consumer. Hardware implementations live in `app-firmware/src/drivers/`; host
   implementations (mocks) live in `app-core/src/hal/mock.rs` (pure `no_std`) and
   `app-mock/src/mocks/` (`std`-dependent, e.g. `HostFilesystem`). These definitions
   must never reference Embassy types, STM32 peripherals, `std`, or board-specific code.
   (Supersedes the former `app-hal` crate, which dissolves in M4.)

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
channel    = "nightly-2025-07-01"   # update this pin periodically
targets    = ["thumbv7em-none-eabihf"]
components = ["rust-src", "llvm-tools-preview", "rustfmt", "clippy"]
```

### Edition
All crates use **`edition = "2024"`** (stabilised in Rust 1.85, February 2025).

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
| AXISRAM | SD card FS state + read buffer | ~16KB |

---

## HAL Traits (currently `app-hal`; moving to `app-core/src/hal.rs` in M4)

`app-hal` currently defines the platform-agnostic audio traits; the new `Filesystem`
trait is added directly in `app-core/src/hal.rs` during M4, and the existing traits
move there at the same time ("ports in the core, adapters at the edges"):

```rust
// Volume/directory abstraction (M4, planned — app-core/src/hal.rs)
pub struct DirEntry {
    pub name: heapless::String<64>,
    pub is_dir: bool,
    pub len: u64,
}

pub trait Filesystem {
    /// Fills `out` with entries of `path`; returns how many. Caller-owned buffer
    /// keeps it alloc-free and embedded-friendly.
    fn read_dir(&mut self, path: &str, out: &mut [DirEntry]) -> Result<usize, FileError>;
}

// Storage abstraction (existing, moves to app-core/src/hal.rs)
pub trait AudioFileReader {
    fn open(&mut self, path: &str)     -> Result<(), FileError>;
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, FileError>;
    fn seek(&mut self, pos: u64)       -> Result<(), FileError>;
    fn file_len(&self)                 -> Result<u64, FileError>;
}

// Audio output abstraction (existing, moves to app-core/src/hal.rs)
pub trait AudioOutput {
    fn write_samples(&mut self, samples: &[i16]) -> Result<(), OutputError>;
    fn sample_rate(&self) -> u32;
    fn channels(&self)    -> u16;
    fn flush(&mut self)   -> Result<(), OutputError>;
}
```

Error enums:
- `FileError { Io, NotFound, SeekError, NotADirectory }` (`NotADirectory` added in M4)
- `OutputError { Overrun, Underflow }`

| Implementation | Location | Description |
|---|---|---|
| `SdCardFilesystem` | `app-firmware/src/drivers/sd_card.rs` (planned, M9+) | `embedded-sdmmc` dir listing for embedded target |
| `SdCardReader` | `app-firmware/src/drivers/sd_card.rs` (planned) | SDMMC file I/O for embedded target |
| `Ak4377Output` | `app-firmware/src/drivers/audio_out.rs` (planned) | SAI + AK4377 audio output |
| `MockFilesystem` | `app-core/src/hal/mock.rs` (M4, planned) | In-memory tree, pure `no_std`, used by `library` unit tests |
| `HostFilesystem` | `app-mock/src/mocks/host_fs.rs` (M4, planned) | `std::fs::read_dir` wrapper for the simulator |
| `MockFileReader` | `app-core/src/hal/mock.rs` (planned) | In-memory file bytes for host tests |
| `MockAudioOutput` | `app-core/src/hal/mock.rs` (planned) | Captures samples for assertions |

> **SD card note:** the workspace pins `embedded-sdmmc 0.7`, whose `DirEntry` exposes a
> `ShortFileName` (FAT 8.3) — long-filename support is version/feature dependent.
> Prefer short-ish filenames when preparing cards, and re-check LFN support (plus the
> 0.7→0.10 `VolumeManager` API changes) at M9 driver bringup.

---

## Display / Simulator (`app-mock`)

- **Pixel colour:** `Gray8` — grayscale pixels. Chosen to match the target monochrome LCD display.
- **Placeholder resolution:** 240×240. Change `DISPLAY_WIDTH` / `DISPLAY_HEIGHT` in
  `ui/display_config.rs` once the real display is chosen.
- **Scale:** `PIXEL_SCALE = 3` (with `PIXEL_SPACING = 0`) — zooms the desktop window to a
  comfortable size.
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
  (`embassy-executor` with `arch-std`, `embassy-time` with `std`), so async task code
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

Host tests never cross-compile (drop `-p app-hal` once M4 dissolves the crate):
```bash
cargo test -p app-hal -p app-core
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
| **Decoder** | `decode_task` (planned) | File path / seek commands from Player | `DecodeError`, buffered PCM |
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
simulator, `mock_player::new()` constructs the `Player` (with a 2-track mock playlist)
and the ui_task performs an initial `refresh_screen` before its event loop starts.

---

## Audio Pipeline (planned target state)

FLAC decoding will be handled by a `Decoder` handler in `app-core` (not yet implemented — Milestone 5):

```
SD card bytes → AudioFileReader → Decoder (Symphonia) → AudioOutput → DAC
```

`Decoder` is platform-agnostic. It calls `AudioFileReader::read()` for input bytes and `AudioOutput::write_samples()` for output PCM. On the host, both sides are mocked. On the STM32, `SdCardReader` and `Ak4377Output` provide real I/O.

The decoder will run in its own async task, filling a bounded PCM buffer. `Player` will consume samples from this buffer each `tick()`, advancing `elapsed_ms` based on actual sample consumption rather than a simulated timer.

**Today**, `Player::tick(delta_ms)` simply advances `elapsed_ms` by a simulated time delta (the desktop simulator's `player_task` injects a 100ms `Command::Tick` via an Embassy `Ticker`), so progress advances even without decoding.

---

## Milestones

- [x] **Milestone 1** Retained-mode GUI framework (`app-core` + `app-mock`) — widget structs (`Label`, `ProgressBar`, `PlayButton`, `List`), screen composition via `Default`, `app-mock` update/sync/draw loop with keyboard navigation
- [x] **Milestone 2** Player state machine + playlist logic (`app-core`, host-tested) — actor model with event-driven state changes, auto-advance, next/prev
- [x] **Milestone 3** Transport controls + progress bar UI (event-driven, auto-advance) — `EventHandler` trait + `PubSubChannel` task bridging running on the real Embassy executor in `app-mock`
- [ ] **Milestone 4** File system + playlist database — dissolve `app-hal` (traits move to
  `app-core/src/hal.rs`), finish the `Track` `heapless::String` ripple, add `Filesystem`
  trait + `DirEntry` + `MockFilesystem` (in-memory, pure `no_std`), write the volume
  scanner in `app-core/src/library.rs` (iterative DFS, extension filter, filename-derived
  titles, `duration_ms = 0` until M5), scan `assets/music/` in the simulator via
  `HostFilesystem` with fallback to the hardcoded playlist
- [ ] **Milestone 5** FLAC decode via Symphonia (desktop end-to-end) — `app-core/src/decoder.rs`, real duration/tag parsing from FLAC STREAMINFO / MP3 headers, host end-to-end test
- [ ] **Milestone 6** Audio mocks — `MockFileReader`/`MockAudioOutput` in `app-core/src/hal/mock.rs` (traits exist by then via M4)
- [ ] **Milestone 7** Hardware bringup (STM32 boot, LED blink, RTT logs)
- [ ] **Milestone 8** AK4377 I2C init + SAI sine wave
- [ ] **Milestone 9** SD card reads + raw PCM playback
- [ ] **Milestone 10** Combine file read and play song on AK4377


## Running Common Tasks

```bash
# Host unit tests (no hardware needed)
# (-p app-hal disappears when M4 dissolves the crate)
cargo test -p app-hal -p app-core

# Run the desktop UI simulator
cargo run -p app-mock

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
