# agents.md — Music Player Firmware

## Project Summary

Embedded Rust portable audio player built on the **STM32H7A3VI** microcontroller using the
**Embassy** async runtime. Audio files are read from an SD card, decoded by **Symphonia**
(MP3 and FLAC), and streamed as PCM over **SAI** to an **AK4377** DAC/headphone amplifier
chip which handles digital-to-analogue conversion and drives the headphones.

The UI and player logic communicate via an **actor-model event system** using
`heapless::Vec<Event, 8>` internal queues. Embassy `Channel` / `Signal` will bridge
these queues between async tasks on the STM32. The desktop simulator validates this
pattern with a single-threaded event cascade.

A desktop **mock** crate mirrors the embedded display using `embedded-graphics-simulator`
so UI code can be developed and tested without hardware.

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

```
music-player-firmware/
├── Cargo.toml                      # Workspace manifest, shared dependency versions
├── rust-toolchain.toml             # Pinned nightly toolchain
├── memory.x                        # STM32H7A3VI linker memory regions
├── .cargo/
│   └── config.toml                 # Workspace-level: [target.thumbv7em-none-eabihf] only
│                                   # NO [build] target here — see app-firmware below
│
├── app-hal/                        # HAL traits + host mock implementations ONLY
│   └── src/
│       ├── lib.rs                  # AudioFileReader, AudioOutput traits
│       └── mock.rs                 # MockFileReader, MockAudioOutput (std only)
│
├── app-core/                       # Pure business logic — no hardware dependencies
│   └── src/
│       ├── event.rs              # Unified Event enum (ButtonPress, Player, Ui)
│       ├── player.rs             # Playback actor — emits events, owns state
│       ├── player/
│       │   └── tests.rs          # Player unit tests (submodule pattern)
│       ├── decoder.rs            # Symphonia FLAC decode (platform-agnostic)
│       ├── decoder/
│       │   └── tests.rs          # Decode unit tests (submodule pattern)
│       ├── playlist.rs           # Fixed-capacity track list
│       ├── playlist/
│       │   └── tests.rs          # Playlist unit tests (submodule pattern)
│       ├── track.rs              # Track metadata (Copy)
│       └── ui/
│           ├── screen_manager.rs # Routes events to active screen
│           ├── screens/
│           │   ├── main_screen.rs
│           │   └── settings_screen.rs
│           └── ...               # Widgets, display config
│
├── app-firmware/                   # Embassy firmware — STM32 target only
│   ├── .cargo/config.toml          # [build] target = "thumbv7em-none-eabihf" ONLY HERE
│   └── src/
│       ├── main.rs                 # Embassy entry point, task spawning
│       ├── drivers/
│       │   ├── sd_card.rs          # SdCardReader: impl AudioFileReader
│       │   ├── audio_out.rs        # Ak4377Output: impl AudioOutput
│       │   └── ak4377.rs           # AK4377 I2C register init sequence
│       └── tasks/
│           ├── decode_task.rs      # Drives Decoder actor
│           ├── player_task.rs      # Drives Player actor
│           └── ui_task.rs          # Button/display handling
│
└── app-mock/                  # Desktop UI simulator — std, runs on host
    ├── .cargo/                     # No config.toml here — inherits host target
    ├── Cargo.toml                  # edition = "2024"
    └── src/
        └── main.rs               # SDL2 + single-threaded event cascade
                                    # Simulates player_task / ui_task roles
```

---

## Architecture Rules

1. **Hardware code lives only in `app-firmware/src/drivers/` and `app-firmware/src/tasks/`.**
   No Embassy types, no STM32 peripheral types anywhere in `app-core` or `app-hal`.

2. **`app-core` and `app-hal` must compile with `cargo test` on the host** (std enabled).
   Never add `embassy-*`, `cortex-m`, or any `no_std`-only dep to these crates without
   also gating it behind `#[cfg(not(test))]`.

3. **All drawing code uses `DrawTarget<Color = Gray8>`** — never reference
   `SimulatorDisplay` directly outside of `app-mock/src/main.rs`. This keeps UI code
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

9. **HAL traits live in `app-hal`; hardware implementations live in `app-firmware`.**
   `app-hal` must never contain Embassy types, STM32 peripherals, or board-specific code.
   It only defines traits and provides `std`-based mock implementations for host testing.

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

## HAL Traits (`app-hal`)

`app-hal` defines platform-agnostic traits used by `app-core` and implemented by hardware drivers in `app-firmware`:

```rust
// Storage abstraction
pub trait AudioFileReader {
    fn open(&mut self, filename: &str)   -> Result<(), FileError>;
    fn read(&mut self, buf: &mut [u8])   -> Result<usize, FileError>;
    fn seek(&mut self, pos: u64)         -> Result<(), FileError>;
    fn file_len(&self)                   -> Result<u64, FileError>;
}

// Audio output abstraction
pub trait AudioOutput {
    fn write_samples(&mut self, samples: &[i16]) -> Result<(), OutputError>;
    fn sample_rate(&self) -> u32;
    fn channels(&self)    -> u16;
    fn flush(&mut self)   -> Result<(), OutputError>;
}
```

| Implementation | Location | Description |
|---|---|---|
| `SdCardReader` | `app-firmware/src/drivers/sd_card.rs` | SDMMC file I/O for embedded target |
| `Ak4377Output` | `app-firmware/src/drivers/audio_out.rs` | SAI + AK4377 audio output |
| `MockFileReader` | `app-hal/src/mock.rs` | `std::fs::File` wrapper for host tests |
| `MockAudioOutput` | `app-hal/src/mock.rs` | Captures samples into `Vec<i16>` for assertions |

---

## Display / Simulator (`app-mock`)

- **Pixel colour:** `Gray8` — grayscale pixels. Chosen to match the target monochrome LCD display.
- **Placeholder resolution:** 240×240. Change `DISPLAY_WIDTH` / `DISPLAY_HEIGHT` in
  `ui/display_config.rs` once the real display is chosen.
- **Scale:** `PIXEL_SCALE = 3` — zooms the desktop window to a comfortable size.
- **Keyboard shortcuts in simulator:**
  - `Space` — toggle play/pause
  - `M` — open Settings
  - `N` — next track
  - `P` — previous track
  - `0`–`9` — seek to 0%–90%
  - `Escape / close` — quit
- **SDL2 required on host:**

- Run with: `cargo run -p app-mock` (from workspace root — uses host target).

---

## GitLab CI/CD Pipeline

Four stages in `.gitlab-ci.yml`:

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
cargo test -p app-hal -p app-core
```

---

## Event Architecture

All inter-module communication uses a unified `Event` enum:

```rust
pub enum Event {
    ButtonPress(Button),            // Input layer (SDL2 / GPIO)
    Player(Playback),               // Commands to Player, or state changes from Player
    Ui(Screen),                     // Screen transitions, refresh requests
}
```

### Actor boundaries

| Actor | Owned by | Receives | Emits |
|---|---|---|---|
| **Player** | `player_task` | `Event::Player(Playback::*)` commands, PCM consumed events | `TrackChanged`, `Toggled`, `ProgressUpdated`, `Stopped` |
| **Decoder** | `decode_task` | File path / seek commands from Player | `DecodeError`, buffered PCM |
| **Active Screen** | `ui_task` | `ButtonPress`, `Player` events | `Player` commands, `Ui(Refresh)`, `Ui(Change)` |
| **ScreenManager** | `ui_task` | `Ui(Change)` | Routes events to active screen |

### Initialization

`Player::initialize()` is called once at startup to emit the initial
`TrackChanged`, `Toggled`, and `ProgressUpdated` events, bootstrapping the
first draw without a special-case initial-sync path.

---

## Audio Pipeline

FLAC decoding is handled by a `Decoder` actor in `app-core`:

```
SD card bytes → AudioFileReader → Decoder (Symphonia) → AudioOutput → DAC
```

`Decoder` is platform-agnostic. It calls `AudioFileReader::read()` for input bytes and `AudioOutput::write_samples()` for output PCM. On the host, both sides are mocked. On the STM32, `SdCardReader` and `Ak4377Output` provide real I/O.

The decoder runs in its own async task, filling a bounded PCM buffer. `Player` consumes samples from this buffer each `tick()`, advancing `elapsed_ms` based on actual sample consumption rather than a simulated timer.

---

## Milestones

- [x] **Milestone 1** Retained-mode GUI framework (`app-core` + `app-mock`) — widget structs (`Label`, `ProgressBar`, `PlayButton`, `List`), screen composition via `Default`, `app-mock` update/sync/draw loop with keyboard navigation
- [x] **Milestone 2** Player state machine + playlist logic (`app-core`, host-tested) — actor model with event-driven state changes, auto-advance, next/prev
- [x] **Milestone 3** Transport controls + progress bar UI (event-driven, auto-advance) — single-threaded event cascade in `app-mock` simulates Embassy tasks
- [ ] **Milestone 4** FLAC decode via Symphonia (desktop end-to-end) — `app-core/src/decoder.rs`, `app-hal` traits, host end-to-end test
- [ ] **Milestone 5** HAL traits + mocks (`app-hal`) — `AudioFileReader`, `AudioOutput`, `MockFileReader`, `MockAudioOutput`
- [ ] **Milestone 6** Hardware bringup (STM32 boot, LED blink, RTT logs)
- [ ] **Milestone 7** AK4377 I2C init + SAI sine wave
- [ ] **Milestone 8** SD card reads + raw PCM playback
- [ ] **Milestone 9** Combine file read and play song on AK4377


## Running Common Tasks

```bash
# Host unit tests (no hardware needed)
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
