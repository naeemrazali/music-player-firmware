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
owner: the `FileManager` handler (its `file_manager_task` wiring is pending — see
*Current status*). Music metadata lives in on-card database files (`music.cat` catalog +
per-field sorted views `title_asc.LST` / `artist_asc.LST`), so RAM usage is O(1) in
library size. The **readers** are implemented; the scan/rebuild path that produces
those files (fingerprint, FNV walk, `MCOMP.CAT` interning) is still planned. See
**SD Library & On-Card Catalog** for the formats and what exists.

The UI, player, and file-access logic communicate via an **actor-model event system**
using `heapless::Vec<Event, 8>` internal queues, formalised by the `EventHandler`
trait. A static Embassy `PubSubChannel` bridges these queues between async tasks —
this is no longer just planned: the desktop simulator runs the real Embassy executor
(`platform-std`) with `ui_task` and `player_task` spawned from
`#[embassy_executor::main]` (`file_manager_task` joins the same pattern in M4), so the
task/channel design is validated on the host before firmware bringup. The same design
carries over to the STM32 firmware tasks.

A desktop **mock** crate mirrors the embedded display using `embedded-graphics-simulator`
so UI code can be developed and tested without hardware.

### Current status

- **Done:** retained-mode GUI framework, `Player` + `Playlist` state machines (actor-model
  pattern, `EventHandler` trait, event queues), transport controls + progress bar UI,
  `app-mock` simulator running the real Embassy executor with a static `PubSubChannel`
  bridging `ui_task` / `player_task`. `app-core` is `no_std` with `extern crate alloc`;
  `hal.rs` holds the `AudioFileReader`/`AudioOutput` traits. `Playlist` is a
  `heapless::Deque<Track, MAX_TRACKS=16>` with `current_index`; `Track` is
  `{title, artist: String<256>, duration_ms: u32, index: usize}`. The M4 event plumbing
  exists end-to-end: `Command::UpdatePlaylist(Box<Playlist>)`,
  `Event::FileManager(Fetch::Playlist { sort_order, start_index, end_index })` with
  `SortOrder` = Title/Artist × Asc/Desc (enum lives in `playlist.rs`). The **M4 read
  path is implemented**: `EventHandler::handle_event` is now a single native
  **`async fn`** (the old sync-plus-`handle_event_async` pair was simplified away),
  and `FileManager<D: BlockDevice<512>>` owns the `FileSystem<D, 512, CACHE_DEPTH=16>`
  plus `TrackList` / `MusicCatalogue` readers (`app-core/src/file_manager/` submodules)
  — `fetch_playlist(sort_order, start, end)` reads `\title_asc.LST` /
  `\artist_asc.LST` + `\music.cat` and emits `UpdatePlaylist`. exfat-slim 0.7 is
  pinned in the workspace (`default-features = false`); the unused `embedded-sdmmc`
  pin is gone. Build is green (one warning: `async fn` in a public trait,
  `event.rs:14`); `cargo test -p app-core -p app-mock` passes (15 tests, all in
  `app-core`).
- **In flight — branch `sorted-list-implementation` (≈70 commits ahead of `main`):**
  work on the M4 fetch path is under way (stages in **Milestones**). The read path
  above is implemented but **nothing drives it yet**: `FileManager::new` builds
  `FileSystem::new(device)` and never calls `mount()` explicitly (exfat-slim
  auto-mounts on first `open`/`write`, so this is a fail-fast gap, not a blocker —
  Stage A adds the call); there is no `file_manager_task` (`EventChannel` is still
  2 pub / 2 sub; `app-mock` has no exfat-slim dependency, no assets, no tests); and
  there is **no scan/rebuild path** — the DB files are assumed to already exist on
  the card, and the header `magic`/`version`/`hash` fields are parsed but never
  validated (see the TODO table). `Player::update_playlist` replaces the playlist
  wholesale (no `window_start`/splice/prefetch logic yet), and `mock_player::new()`
  still seeds a hardcoded 2-track playlist.
- **Next up — Stage A prototype (agreed plan, not yet implemented):** the happy-path
  FS-actor + sim-volume stage (details in **Milestones** → Stage A) —
  `file_manager_task` mounts an in-RAM exFAT volume and publishes the boot window
  `[0, 16)` so `cargo run -p app-mock` plays a real (hand-seeded) library end-to-end.
  Locked decisions: the volume is exfat-slim's own example image, committed at
  `app-mock/assets/exfat.img.gz` (mkfs tooling deferred to Stage D); gunzip uses
  `flate2` (miniz_oxide has no gzip-container API; flate2's default backend IS
  miniz_oxide); the DB files are hand-serialised by `populate_db()`; every failure
  site stays a `// TODO(errors)` comment (happy-path-first policy).
- **Decided for M4 (locked design, partially implemented):** exFAT-only storage via
  exfat-slim 0.7.0 with `default-features = false` (no alloc, no logging, **no embassy**
  — the crate's built-in fs actor was evaluated and rejected; see *exfat-slim notes*).
  **`file_manager_task` is the FS actor**: its `FileManager` handler is the sole owner
  of the raw `exfat_slim::asynchronous::FileSystem`, so "streaming a music file while
  opening a sorted list" is one task interleaving at await points — no FS sharing, no
  locks. Still planned on top of the landed readers: the build side of the on-card
  database (FNV-1a generation fingerprint scanned every boot; `MCOMP.CAT` string
  interning; `Track.index` → `cat_index: u32` pointing at its `.CAT` record) and the
  externally driven `Playlist` window (boot fetch `[0,16)` + in-playback prefetch;
  `Player` splices on apply and prefetches near the window edge). The simulator will
  mount a committed gzipped exFAT image in RAM and populate it through exfat-slim's
  own write path (DB seeds first — Stage A; CJK samples land with Stage D). See
  **SD Library & On-Card Catalog**, **Event Architecture**, and **Milestones**.
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
  contained to the `app-core/src/file_manager/` submodules and the `BlockDevice`
  adapters.

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
│                                   #   (exfat-slim = "0.7" pinned; embedded-sdmmc pin
│                                   #    removed; Stage A adds aligned + flate2)
├── rust-toolchain.toml             # Pinned nightly toolchain (nightly-2026-04-11)
├── memory.x                        # (planned, not yet created) linker memory regions
├── .cargo/
│   └── config.toml                 # `[target.thumbv7em-none-eabihf]` runner + rustflags only
│                                   # NO [build] target here — see app-firmware below
├── tools/
│   └── make_exfat_image.sh          # (Stage D, planned) mkfs.exfat a small volume, gzip it
│                                   #   → app-mock/assets/exfat.img.gz (until then the asset
│                                   #   is a copy of exfat-slim's own example image)
│
├── app-core/                       # Pure business logic — no hardware, no_std + alloc
│   └── src/
│       ├── lib.rs                  # Module declarations (#![no_std], extern crate alloc)
│       ├── event.rs                # Button, Command, State, Screen, Fetch, Event enums +
│       │                           #   EventHandler trait (handle_event is a native
│       │                           #   async fn) + EventQueue type
│       ├── player.rs               # Player handler — implements EventHandler, owns state
│       ├── player/
│       │   └── tests.rs            # Player pure-logic tests (12)
│       ├── playlist.rs             # Deque<Track,16> + current_index + SortOrder enum;
│       │                           #   (Stage B, planned) window_start + splice methods
│       ├── playlist/
│       │   └── tests.rs           # Playlist pure-logic tests (3)
│       ├── track.rs                # {title, artist: String<256>, duration_ms: u32,
│       │                           #   index: usize = catalogue record index};
│       │                           #   (Stage B, planned) index → cat_index: u32
│       ├── file_manager.rs         # FileManager<D: BlockDevice<512>> handler — owns the
│       │                           #   FileSystem + TrackList + MusicCatalogue;
│       │                           #   fetch_playlist(sort_order, start, end) → Playlist
│       │                           #   (⚠ never calls fs.mount() explicitly yet; (Stage A)
│       │                           #   new(fs) takes the constructed FileSystem)
│       ├── file_manager/
│       │   ├── filesystem_io.rs    # Fs<D> = FileSystem<D, 512, CACHE_DEPTH=16>, FsError
│       │   │                       #   (NotFound | Io), open_read(), read_at()
│       │   │                       #   (seek + read), exfat_err_to_fs_err mapping
│       │   ├── header.rs           # Shared 24B DB header {magic, version, num_items,
│       │   │                       #   hash: u64, reserved} — parsed, NOT validated yet
│       │   ├── helpers.rs          # U32Buf/U64Buf type aliases, parse_u32_le/parse_u64_le
│       │   ├── music_catalogue.rs  # \music.cat reader — 580B records → Track
│       │   │                       #   (title[256] @0, artist[256] @256, 64B reserved
│       │   │                       #   region @512, duration_ms u32 @576)
│       │   └── track_list.rs       # \title_asc.LST / \artist_asc.LST reader — u32
│       │                           #   catalogue indices; Descending = reversed
│       │                           #   index (convert_to_actual: num_items-1-i)
│       ├── hal.rs                  # AudioFileReader, AudioOutput traits + error enums
│       │                           #   (NO Filesystem trait — exfat-slim IS the shared FS
│       │                           #    layer, used directly in file_manager/)
│       └── ui/
│           ├── ui.rs               # Widgets live here: Label, ProgressBar, PlayButton,
│           │                       #   HorizontalLine, List, clear_background, fmt_time_ms
│           ├── display_config.rs   # DISPLAY_WIDTH/HEIGHT (240×240), PIXEL_SCALE (3)
│           ├── screen_names.rs     # ScreenName enum (Main, Settings)
│           ├── screen_manager.rs   # EventHandler impl routing events to active screen
│           └── screens/            # Screen structs (MainScreen, SettingsScreen), each
│               ├── screens.rs      #   module file; every screen is an EventHandler
│               ├── main_screen.rs  #   with its own event queue
│               └── settings_screen.rs
│
├── app-firmware/                   # Embassy firmware — STM32 target only
│   ├── .cargo/config.toml          # [build] target = "thumbv7em-none-eabihf" ONLY HERE
│   └── src/
│       ├── main.rs                 # (placeholder std hello-world — no dependencies yet;
│       │                           #   does not build for the embedded target)
│       └── drivers/sd_card.rs      # (M9, planned) exfat-slim BlockDevice over embassy-stm32
│                                   #   SDMMC + MBR type-0x07 partition-offset translation
│
└── app-mock/                       # Desktop simulator — std, runs real Embassy executor
    ├── Cargo.toml                  # edition = "2024"; (Stage A, planned) + exfat-slim,
    │                               #   aligned, flate2 (gunzip)
    ├── assets/
    │   ├── exfat.img.gz            # (Stage A, planned) committed exFAT volume — copy of
    │   │                           #   exfat-slim 0.7.0's own example image (17KB gz /
    │   │                           #   10MB raw, Apache-2.0, attribution in sd_image.rs);
    │   │                           #   self-regeneration deferred to Stage D
    │   └── music/                  # (Stage D, planned) sample files + CJK names — the DB
    │                               #   format doesn't reference them until M5 reopen
    └── src/
        ├── main.rs                 # #[embassy_executor::main], static PubSubChannel,
        │                           #   spawns ui_task + player_task (+(Stage A, planned)
        │                           #   file_manager_task — spawn order matters, see
        │                           #   Event Architecture)
        ├── tasks.rs                # Task<T: EventHandler> struct (owns handler + pub/sub
        │                           #   endpoints; awaits the async handle_event),
        │                           #   EventChannel (2 pub / 2 sub today; 3/3 in Stage A)
        ├── tasks/
        │   ├── player_task.rs      # Player task — services Player commands + 100ms tick;
        │   │                       #   (Stage A) Player::new(empty Playlist), windows
        │   │                       #   arrive via UpdatePlaylist
        │   ├── ui_task.rs          # UI task — keyboard input, screen refresh loop
        │   └── file_manager_task.rs # (Stage A, planned) THE FS ACTOR TASK — device +
        │                           #   FileSystem → mount → populate_db (sim seed) →
        │                           #   boot fetch [0,16) → publish → loop on
        │                           #   Event::FileManager(_)
        ├── mocks.rs                # mock_display, mock_window, mock_player constructors
        ├── mocks/
        │   ├── mock_display.rs
        │   ├── mock_player.rs      # hardcoded 2-track playlist today; (Stage A) switches
        │   │                       #   to Player::new(empty) — the real library arrives
        │   │                       #   via UpdatePlaylist
        │   └── sd_image.rs         # (Stage A, planned) ExFatRamBlockDevice
        │                           #   (include_bytes! + flate2 GzDecoder → Vec; impl
        │                           #   async BlockDevice<512>, Align = A4, Error =
        │                           #   Infallible, multi-block read/write, size()) +
        │                           #   populate_db(): hand-serialised music.cat / .LST
        │                           #   seeds via FileSystem::write; (Stage D)
        │                           #   populate_from_dir + CJK samples
        └── tests.rs                 # (Stage A: smoke tests — fetch ordering asc/desc via
                                     #   direct handle_event calls; Stage E: grows into a
                                     #   tests/ module — corrupt/stale headers, window
                                     #   slides, player e2e with prefetch, reopen-by-CJK-
                                     #   path). The image tooling is simulator-only, so all
                                     #   tests that need a mounted volume live in app-mock
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
    no other task touches the FS. The `app-core/src/file_manager/` submodules call the
    raw async API (`open` with `OpenOptions`, `File::read/seek` taking
    `&mut FileSystem`; the planned builder will add `read_dir` with a caller-supplied
    765B name buffer and `write`). The crate's `embassy` feature (internal
    request-queue actor) is **rejected** — see *exfat-slim notes*. `app-mock` will
    supply `ExFatRamBlockDevice` (gunzipped committed image); `app-firmware` supplies
    the embassy-stm32 SDMMC adapter **plus MBR partition-offset translation**
    (exfat-slim mounts at view-sector 0; real SDXC cards carry an MBR with a
    type-0x07 exFAT partition whose LBA the adapter must add to every sector address
    — M9). `hal.rs` keeps `AudioFileReader`/`AudioOutput` as the M5 decoder's ports.
    (Supersedes the earlier embedded-sdmmc/FAT32 plan and the `Filesystem` trait.)

10. **The on-card database files are private formats; the fingerprint is the glue.**
    Any byte layout change to `music.cat` / the `*.LST` views (and, once it exists,
    `MCOMP.CAT`) bumps its `version` field (old files auto-rebuild when the
    scan/rebuild path lands). All files of one generation share one fingerprint;
    never mix generations.

11. **`EventHandler::handle_event` is a native `async fn` in the trait (AFIT).**
    The earlier sync-`handle_event` + `handle_event_async`-default pair was simplified
    into one async trait method: every handler implements it, and `Task::service_event`
    awaits it. Handlers with no async needs (Player, screens) implement the async
    signature and complete without awaiting inside; `FileManager` awaits its FS reads.
    This currently emits the "`async fn` in public traits is discouraged" warning
    (auto trait bounds cannot be specified — `event.rs:14`), accepted on the pinned
    nightly — the same pattern exfat-slim itself uses for its `BlockDevice` trait.

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
  (see *SD Library & On-Card Catalog* → *exfat-slim notes*). Landed in the workspace
  with `default-features = false`, and it stays that way: no logging, no alloc, no
  embassy. The `embassy` feature (crate-internal request-queue actor: `fs_actor_task` +
  `FileHandle` client API; implies `alloc`, `Vec`-per-read copies) was **evaluated and
  rejected** — `file_manager_task` owns the raw `FileSystem` instead (rule 9). With all
  features off the dep tree is `aligned`, `bitflags`, `block-device-driver`,
  `byteorder`, `exfat-slim-macros`, `heapless`, `thiserror` — all `no_std`, zero
  `embassy-*` crates in `app-core`.
- `flate2` in `app-mock` only — (Stage A, planned) gunzips `assets/exfat.img.gz` at
  startup via `GzDecoder`. Chosen over raw `miniz_oxide`, which exposes only
  raw-DEFLATE/zlib `decompress_to_vec*` (no gzip-container API); flate2's default
  backend *is* `miniz_oxide`, and exfat-slim's own examples use `GzDecoder` on this
  exact asset. (Supersedes the earlier miniz_oxide plan.)
- `aligned` in `app-mock` — (Stage A, planned) provides `A4`/`Aligned` for the
  `ExFatRamBlockDevice` `BlockDevice` impl (same crate/version exfat-slim uses).
- The unused `embedded-sdmmc` workspace pin is **gone** (removed with the M4 Stage 0
  dep cleanup; no crate referenced it).

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
| AXISRAM | `FileManager` state + `FileSystem` sector caches (**3 × N=16 × 512B**: data + FAT + alloc-bitmap) + `Deque<Track,16>` window (16 × ~520B) | ~33KB |
| AXISRAM | LST build sort buffer `Vec<Track, CAP=256>` (~520B/entry) | ~148KB, boot-transient only |
| AXISRAM | DFS walk buffers (765B name buf, ~4KB path prefix, on the file_manager_task stack) | ~5KB, boot-transient only |

**Display framebuffer (provisional estimate):** a full 240×240 Gray8 frame is
57.6KB — the safe upper bound matching the `DrawTarget<Color = Gray8>` rule, so the
UI code's colour model carries to firmware unchanged. SRAM1 (64KB, otherwise
unbudgeted) hosts it with ~6KB spare; AXISRAM is the fallback (steady-state usage
is ~129KB of 1024KB). Alternatives once the panel is chosen: a 1bpp buffer (7.2KB,
needs draw-time or flush-time conversion) or line/window streaming (~0 RAM,
controller-dependent). No double-buffer — the retained-mode UI redraws on event
only. Final placement also depends on the display controller's DMA paths (DTCM is
excluded for DMA, rule 4; SRAM1/AXISRAM are both reachable).

The static `PubSubChannel<Event, 64>` stays small: `State::TrackChanged(Option<Box<Track>>)`
and `Command::UpdatePlaylist(Box<Playlist>)` box the big payloads, so each slot is a few
dozen bytes.

---

## SD Library & On-Card Catalog (M4 design — read side implemented)

RAM usage is O(1) in library size: the sorted lists live **on the SD card**. Three
private files in the volume root — `\music.cat` (catalog), `\title_asc.LST` /
`\artist_asc.LST` (per-field sorted views). The **read side is implemented** in
`app-core/src/file_manager/`; the **build side (scan / fingerprint / rebuild) is still
planned** — nothing in the repo creates these files, so the card is assumed to carry a
current, well-formed database (headers are parsed but never validated; see the TODO
table). A fourth file (`MCOMP.CAT`, interned component table for path reconstruction)
is planned alongside the M5 reopen path; the 64 B region in each CAT record is reserved
for it.

### `\music.cat` — unsorted catalog (scan order)

Header **24 B** (shared layout, parsed by `header.rs::Header` — *infallible, not
validated yet*): 4B magic | 4B version (u32 LE) | 4B count (u32 LE) | 8B fingerprint
(u64 LE) | 4B reserved (u32 LE). (The design calls the last field `skipped_count`; the
reader parses it as an unused `reserved` until the scanner exists.)

Records **580 B** each (record *i* at `24 + i×580`; no truncation check yet):

| Offset | Size | Field | Encoding |
|---|---|---|---|
| 0 | 256 | `title` | UTF-8 NUL-padded — filename stem (CJK intact); M5 overwrites with parsed tags |
| 256 | 256 | `artist` | UTF-8 NUL-padded — `""` until M5 |
| 512 | 64 | *(reserved)* | planned `chain[16]` u32 interned-component indices → `MCOMP.CAT` (unread today) |
| 576 | 4 | `duration_ms` | u32 LE — size-based estimate at scan (~192 kbps MP3, ~900 kbps FLAC) until M5 |

### `\title_asc.LST` + `\artist_asc.LST` — sorted views (u32 catalog indices)

One file per sort field, header **24 B** each (same shared layout; the design puts
`magic "MLST"` | version | order | count | fingerprint there — the reader only uses
`count`). Body: `count × u32 LE` — `LST[i]` = catalog record index of logical
position *i*; file length `24 + count×4`. `SortOrder` has four variants
(Title/Artist × Asc/Desc) but only two files exist: **Descending = reverse iteration
of the same file** — `TrackList::convert_to_actual` maps logical *i* → physical
`num_items - 1 - i`; no second LST. Artist order is degenerate until M5 tag parsing
fills real `artist` fields (all `""` ⇒ stable catalog order) — accepted. Future
orders share this layout.

### Fingerprint & lifecycle (planned — not implemented)

- **Fingerprint** = FNV-1a u64 over every *eligible* file's `(path, size)` — a DFS
  will walk the card hashing incrementally each boot (O(1) RAM, no writes). Skipped
  files are invisible to it (eligible-set semantics: a skipped file changing never
  triggers a rebuild). u64 keeps the silent-miss probability fleet-negligible. Today
  the header `hash` field is parsed but ignored.
- **Boot (planned)**: mount → header validation (`magic` → `version` →
  `count`-vs-file-length) → fingerprint walk → mismatch/missing ⇒ rebuild: one DFS
  pass streaming unsorted records into a fresh CAT (and appending `MCOMP.CAT`
  entries in the same pass, once interning lands) → LST load-or-build (fingerprint
  match ⇒ serve; else read CAT sequentially, sort into `Vec<Track, CAP=256>`, write).
  An unchanged card never sorts. **None of this exists yet** — `FileManager::new`
  builds `FileSystem::new(device)` and never calls `mount()`; the boot sequence
  belongs to `file_manager_task` (see **Event Architecture**).
- **Skip policies (planned)** (`skipped_count` in the CAT header): chains deeper than
  16 components; reconstructed paths exceeding the ~4KB path budget (bounds the M5
  reopen buffer). Component names can never overflow (765 B is the format max).
- **Access (implemented)**: `FileManager::fetch_playlist(order, start, end)` =
  (re)open LST → seek to `24+i×4` (reverse mapping for Descending via
  `convert_to_actual`) → read `(end-start)` indices → for each, seek to
  `24+idx×580` and read the 580 B CAT record → hydrate `Track` — fixed-offset reads
  riding exfat-slim's slot cache, awaited inside `FileManager::handle_event`. Note:
  both the LST and the CAT are **re-opened on every `fetch_playlist` call** (a
  cache-the-handles TODO — the readers hold `Option<File>` + parsed header precisely
  so they can be reused).
- **Reopen (M5, planned)**: record → ≤16 MCOMP reads → reconstruct
  `"/Music/日本語/曲.flac"` → open **via a request to `file_manager_task`** (sole FS
  owner, rule 12) — once per song.
- **`Track` (RAM)**: `{title: String<256>, artist: String<256>, duration_ms: u32,
  index: usize}` (~520 B) — `index` is the CAT record index today; the
  `cat_index: u32` rename lands with the window work. The path exists only as the
  (future) on-card chain.

### Playlist window (externally driven — M4 decision, not yet implemented)

The FS reads will **not** live in `Playlist`; the window is fed over the event
channel. Plan: `Playlist` = `heapless::Deque<Track, MAX_TRACKS=16>` + absolute
logical `window_start` + relative `current_index` (absolute current = `window_start +
current_index`). `MAX_STRING_LENGTH = 256` drives title/artist/`Label`.

- **Fetch protocol**: `Player` emits `Event::FileManager(Fetch::Playlist { sort_order,
  start_index, end_index })`; `file_manager_task` awaits the fetch and publishes
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
- **End of library**: the fetch clamps past-the-end ranges; when the window truly
  exhausts, `next()` → `None` ⇒ `Player::stop()` (existing semantics preserved).
- **Today**: none of the window logic exists — `Playlist` is a plain `Deque` +
  `current_index` (no `window_start`), `Player::update_playlist` replaces the list
  wholesale, and no prefetch/`fetch_in_flight` guard is emitted.

### exfat-slim notes (verified against 0.7.0 source)

- Path-based API (`open`, `read_dir`, `create_directory`, `write`, `exists`, `remove_file`,
  `rename`), blocking and async from one source; `OpenOptions` with
  create/truncate/append; `seek(u64)`; full **write** support. Paths accept both `\`
  and `/` separators (split on `['/', '\\']`), so the readers' `\\music.cat`-style
  root paths work.
- `FileSystem::write(path, impl AsRef<[u8]>)` is an alloc-free convenience (create +
  truncate + auto-mkdir + close in one call) — `populate_db` builds each DB file in
  a host `Vec` and writes it via a single `fs.write`.
- `mount()` is idempotent and auto-invoked on first `open`/`write` — an explicit
  `mount()` at task start stays the convention for fail-fast (Stage A adds it).
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
  mirrors this. The FS holds **three** sector caches of N slots each (data, FAT,
  allocation bitmap) — RAM cost is `N × 3 × 512B`, not `N × 512B`. The README
  recommends `N = 4` (6KB) for embedded; we pick `N = 16` (24KB) for two-consumer
  headroom (streaming file + list-fetch sectors share the data cache).
- `mount()` expects the exFAT boot sector at view-sector 0 — adapters must apply the
  MBR partition LBA themselves.
- Known TODOs (accepted risks): reads past `valid_data_length` don't zero-fill (our
  reads are always exact-offset/exact-length); no double-open file locking (single
  ownership in `file_manager_task` makes it moot — re-verify when M5 decoder reads
  interleave in the same task). Mitigations: pinned version, FS layer isolated in
  `file_manager.rs` + the `file_manager/` submodules, the app-mock behavioral suite,
  and a documented swap path back to embedded-sdmmc if the crate disappoints.

### Simulator image strategy (planned — Stage A, decisions locked)

`app-mock` embeds the committed `assets/exfat.img.gz` via `include_bytes!` (no
runtime file IO, no missing-asset fallback needed) and gunzips it with `flate2`
into a `Vec`-backed `ExFatRamBlockDevice` at task start. The initial asset is a
copy of exfat-slim 0.7.0's own example volume (`examples/common/sd.img.gz`, 17KB
gzipped / 10MB raw, Apache-2.0 — attribution comment in `sd_image.rs`);
self-regeneration via `tools/make_exfat_image.sh` is deferred until volume-size
control matters (Stage D). `populate_db()` then writes the DB files through
exfat-slim's own `FileSystem::write`: hand-serialised `music.cat` (magic `"MLIB"`,
version 1, hash 0 — TODO Stage C fingerprint) and the two `.LST` views (magic
`"MLST"`), seeded with 5 ASCII tracks (CJK seeds wait for the CJK font work — the
format round-trips UTF-8 fine, but `FONT_6X10` cannot render it). Actual music
files land with Stage D's `populate_from_dir` — the DB format doesn't reference
them until the M5 reopen path.

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
    async fn handle_event(&mut self, event: &Event) -> EventQueue;
    fn push_events(&mut self) -> EventQueue;  // drains the internal queue
    fn add_event(&mut self, event: Event);
}
```

`Task::service_event` awaits `handle_event` — it is now a native AFIT method, with no
sync variant and no separate `handle_event_async` (rule 11). Handlers with no async
needs (Player, screens) still implement the async signature and complete without
awaiting inside; `FileManager` awaits its FS reads. This shape currently emits the
"`async fn` in public traits is discouraged" warning (auto trait bounds cannot be
specified) — accepted on the pinned nightly.

### Handler boundaries

Handlers implement `EventHandler`; the "actor model" is the communication pattern
(own state + message passing via event queues) they follow.

| Handler | Owned by | Receives | Emits |
|---|---|---|---|
| **Player** | `player_task` | `Event::Player(Command::*)` incl. `UpdatePlaylist`, 100ms `Tick` | `Playback(State::*)`: `TrackChanged`, `Toggled`, `ProgressUpdated`, `Stopped`; *(planned)* `FileManager(Fetch::Playlist)` prefetch near the window edge |
| **FileManager** | `file_manager_task` *(planned — no task yet)* | `Event::FileManager(Fetch::*)` | `Player(Command::UpdatePlaylist)` — **sole owner of the exFAT `FileSystem`** (its `mount()` call is pending, see *Current status*) |
| **Decoder** | `decode_task` (M5) | catalog index / seek commands from Player; streams its file **via `file_manager_task`** (mechanism TBD at M5) | `DecodeError`, buffered PCM |
| **Active Screen** | `ui_task` | `ButtonPress`, `Playback(State::*)` | `Player` commands, `Ui(Refresh)`, `Ui(Change)` |
| **ScreenManager** | `ui_task` | `Ui(Change)` | Routes events to active screen |

### Task bridging (app-mock, and later app-firmware)

- A static `PubSubChannel<CriticalSectionRawMutex, Event, 64, 2, 2>` (`EventChannel` in
  `app-mock/src/tasks.rs` — 2 subscribers / 2 publishers today; counts go to 3/3 in M4
  when `file_manager_task` joins) is shared by all tasks. Each task wraps its handler
  in the generic `Task<T: EventHandler>` struct, which owns a publisher + subscriber
  endpoint, awaits the async `handler.handle_event()`, then publishes the resulting
  events.
- **player_task** selects between `subscriber.next_message()` and a 100ms `Ticker`.
  It services `Event::Player(_)` commands and injects `Command::Tick(100)` on timeout.
  It publishes everything its handler emits — including *(planned)*
  `FileManager(Fetch::Playlist)` prefetch requests.
- **ui_task** selects between `subscriber.next_message()` and a 33ms timeout. It
  services `Playback(_)` state changes (which schedule `Screen::Refresh`) and
  `Ui(Screen::Change)`, and polls SDL2 for keyboard input on timeout.
- **file_manager_task** *(M4, planned — the FS actor)*: at startup it constructs
  `FileSystem::new(ExFatRamBlockDevice)` → `mount()` → *(sim only)* `populate_db()`
  seed write → hands the `FileSystem` to `FileManager::new(fs)` → boot fetch
  `[0, 16)` → publishes `UpdatePlaylist`; then it loops on
  `subscriber.next_message()` servicing `Event::FileManager(_)`. It never touches
  the `Event::Player(_)` / `Playback(_)` traffic.
- **Spawn order matters (M4):** `ui_task` → `player_task` → `file_manager_task`.
  Embassy pubsub drops messages published before a subscriber joins, and `Task::new`
  subscribes synchronously at task start, so earlier-spawned tasks are guaranteed
  subscribed before `file_manager_task`'s first publish (its boot scan awaits, which
  yields to them first). Document this in `main.rs`. (Today only `ui_task` +
  `player_task` are spawned.)
- **Event echo control:** each task filters which channel messages it services, so
  events published by a task are not re-processed by it (no infinite loops).

### Initialization

`Player::new(playlist)` emits `TrackChanged`, `Toggled(true)`, and `ProgressUpdated`
events into its internal queue at construction, bootstrapping the first UI draw
without a special-case initial-sync path. The plan (M4): the constructor is called
with an **empty** `Playlist` — the emitted `TrackChanged(None)` seeds the UI while
`file_manager_task` mounts, fetches the first window `[0, 16)`, and publishes
`UpdatePlaylist`; playback of track 0 begins on arrival. The ui_task performs an
initial `refresh_screen` before its event loop starts. (Today, pending
`file_manager_task`, `mock_player::new()` still seeds a hardcoded 2-track playlist.)

---

## Audio Pipeline (planned target state)

FLAC/MP3 decoding will be handled by a `Decoder` handler in `app-core` (not yet implemented — Milestone 5):

```
SD card bytes → AudioFileReader → Decoder (Symphonia) → AudioOutput → DAC
```

`Decoder` is platform-agnostic. It reopens a track via its catalog record →
*(planned)* `MCOMP.CAT` chain → reconstructed path, then streams the file **through
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
- [ ] **Milestone 4** SD library + playlist database (exFAT — in progress on branch
  `sorted-list-implementation`; see **SD Library & On-Card Catalog**). Landed already:
  `Deque<Track,16>` playlist, `MAX_STRING_LENGTH = 256`, `Fetch::Playlist` /
  `UpdatePlaylist` event plumbing, 4-variant `SortOrder` (lives in `playlist.rs`),
  exfat-slim 0.7 pinned (embedded-sdmmc pin removed), single async
  `EventHandler::handle_event`, and the **read side of the fetch path**:
  `FileManager<D>` owning `FileSystem<D, 512, 16>` + `TrackList`/`MusicCatalogue`
  readers (`file_manager/` submodules: `filesystem_io`, `header`, `helpers`,
  `music_catalogue`, `track_list`) with `fetch_playlist(sort_order, start, end)`
  implemented. Remaining work (stages, in order):
  - **Stage A — FS actor + sim volume + smoke tests (agreed plan — next up):**
    app-core: `#[allow(async_fn_in_trait)]` on `EventHandler` (the documented
    accepted warning — restores a green `clippy --workspace -- -D warnings` gate);
    `Player::update_playlist` emits `TrackChanged` + `ProgressUpdated` after
    replace (without it the UI never shows the boot track — `// TODO(Stage B)`:
    splice/prefetch semantics); `FileManager::new(fs: Fs<D>)` takes the constructed
    `FileSystem` (the task owns construction + mount + sim populate, then hands
    over sole ownership). app-mock: deps exfat-slim + aligned + flate2 (workspace
    pins); `assets/exfat.img.gz` (copy of exfat-slim 0.7.0's example image,
    Apache-2.0, attribution in `sd_image.rs`); `mocks/sd_image.rs` —
    `ExFatRamBlockDevice` (`include_bytes!` + `GzDecoder` → `Vec`,
    `BlockDevice<512>`, `Align = A4`, `Error = Infallible`, multi-block
    `read`/`write`, `size()`) + `populate_db` (hand-serialised CAT/LST via
    `FileSystem::write`: magic `MLIB`/`MLST`, version 1, hash 0, 5 ASCII seed
    tracks, 64B chain region zeroed; boot window hardcodes 16 pending exposing
    `playlist::MAX_TRACKS`); `tasks/file_manager_task.rs` (mount → populate →
    boot fetch `[0, 16)` `TitleAscending` via `service_event` → loop on
    `Event::FileManager(_)`); `EventChannel` counts 2→3 pub / 3 sub; spawn order
    ui → player → file_manager; `mock_player::new()` → `Player::new(empty)`;
    smoke tests in `app-mock/src/tests.rs` (title-asc, artist, desc reversal —
    direct `handle_event` calls with `embassy_futures::block_on`, no SDL).
    Happy-path-first: every failure site is a `// TODO(errors)` (see error table).
  - **Stage B — playlist window:** `Track.index: usize` → `cat_index: u32`;
    `Playlist.window_start` + splice methods; `Player` window logic (prefetch emit at
    `current_index + 2 >= len` behind a `fetch_in_flight` guard, splice-or-replace on
    `UpdatePlaylist`); cache `TrackList`/`MusicCatalogue` handles across fetches
    (today both DB files are re-opened on every `fetch_playlist` call); update
    player/playlist tests
  - **Stage C — build side (scan/rebuild):** the boot DFS + FNV-1a fingerprint +
    header validation + CAT/LST rebuild pipeline (*Fingerprint & lifecycle* above),
    incl. `MCOMP.CAT` interning and `skipped_count` accounting; pure-logic tests in
    `app-core/src/file_manager/tests.rs` (header/record codecs, FNV vectors,
    comparator, chain→path)
  - **Stage D — sim tooling remainder:** `tools/make_exfat_image.sh` (mkfs.exfat
    self-regenerated volume — replaces the copied example image when size control
    is needed); `populate_from_dir` with CJK-named samples **and real music files**
    (the DB format doesn't reference them until M5 reopen)
  - **Stage E — behavioral suite:** the rest of `app-mock/src/tests/`:
    corrupt/stale headers, window slides, player e2e with prefetch firing
    during playback, reopen-by-CJK-path once MCOMP lands
- [ ] **Milestone 5** FLAC decode via Symphonia (desktop end-to-end) — `app-core/src/decoder.rs`, real duration/tag parsing from FLAC STREAMINFO / MP3 headers written back into catalog records, reopen-by-chain, **music-file streaming through `file_manager_task`** (sole FS owner; request mechanism TBD — see **Audio Pipeline**), host end-to-end test
- [ ] **Milestone 6** Audio mocks — `MockFileReader`/`MockAudioOutput` in `app-core/src/hal/mock.rs` (traits exist in `hal.rs`)
- [ ] **Milestone 7** Hardware bringup (STM32 boot, LED blink, RTT logs)
- [ ] **Milestone 8** AK4377 I2C init + SAI sine wave
- [ ] **Milestone 9** SD card reads + raw PCM playback — exfat-slim `BlockDevice` adapter
  over embassy-stm32 SDMMC **plus MBR type-0x07 partition-offset translation**
- [ ] **Milestone 10** Combine file read and play song on AK4377

---

## Error-Handling & Robustness TODOs (deferred — happy-path-first policy)

M4 builds the architecture assuming well-formed private files and a present card:
error detection is deliberately deferred so the fetch path lands end-to-end first.
Gaps are marked `// TODO(...)` in code and tracked here. Close these before M5
leans on the same paths. **Note:** the boot fingerprint rebuild is the main safety
net in the design — but it does not exist yet, so today there is *no* protection at
all: headers are parsed but not validated, and a stale or garbage DB file decodes
nonsense mid-session undetected until items 1–3 land.

| # | Gap | Where | Planned fix |
|---|---|---|---|
| 1 | DB headers not validated — no magic/version check (TODO in both `open()`s); a stale generation decodes a garbage `num_items` | `TrackList::open` (`track_list.rs`) + `MusicCatalogue::open` (`music_catalogue.rs`); shared infallible `Header::parse` (`header.rs`) | `parse` becomes fallible ⇒ dedicated `Corrupt` variant ⇒ treated like a stale file (boot rebuild once Stage C exists) |
| 2 | `index >= num_items` maps to `FsError::Io` | `TrackList::fetch_track_index` + `MusicCatalogue::fetch_track` | dedicated `OutOfRange` variant (range clamping stays in `fetch_playlist`) |
| 3 | EOF-mid-fill / seek-past-EOF flattened into `Io` (TODO in `read_at`) | `filesystem_io::read_at` | split out `Truncated` |
| 4 | `FsError` drops the device error detail | `filesystem_io::exfat_err_to_fs_err` | optional `code: u32` when defmt logging lands (M7+) |
| 5 | CAT / LST file-length + truncation checks (`24 + count×len == file_len`) | catalog + track-list readers | same pattern as #1; rule 10 version bumps |
| 6 | Boot-scan skip accounting (`skipped_count`, unreadable entries) | future builder (M4 Stage C — no scan exists yet) | per skip-policies design |
| 7 | Card removal / swap at runtime (NoCard ⇒ unmount; remount ⇒ fingerprint rescan) | `FileManager` (M9) | `Option<FileSystem>` + device recovery via `unmount(self) -> D`; mount stays eager at task start |
| 8 | Reopen-path bounds (chain ≤ 16 components, ~4KB path budget) | M5 decoder reopen | per skip-policies design |
| 9 | Fetch failure ⇒ player/UI behaviour — today a failed index is silently skipped (`continue`) and a short/empty playlist is still emitted | `FileManager::fetch_playlist` / Player apply logic | graceful degradation states |
| 10 | Sim startup failures — corrupt/missing embedded asset (gunzip `unwrap`), mount failure, populate write failure all panic at task start (`expect`) | `tasks/file_manager_task.rs` + `mocks/sd_image.rs` (simulator only) | "no library" degradation state shared with #7/#9 |

---

## Running Common Tasks

```bash
# Host unit tests (no hardware needed; 15 tests in app-core today —
# app-mock's behavioral suite lands with M4 Stage E)
cargo test -p app-core -p app-mock

# Run the desktop UI simulator
cargo run -p app-mock

# Regenerate the committed empty exFAT image (planned — tool not written yet;
# requires mkfs.exfat from exfatprogs)
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
