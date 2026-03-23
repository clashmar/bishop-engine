# Audio System Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement music + SFX playback with master/music/sfx volume groups exposed to Lua, built on a portable fire-and-poll threading model.

**Architecture:** Two contracts in `engine_core/src/task/` enforce fire-and-poll at the type level (`BackgroundTask<T>` for one-shot work, `BackgroundService` for persistent services). The audio backend is abstracted behind `AudioBackend` in `bishop/src/audio/` so cpal is a feature-gated desktop default — console backends slot in without changing engine_core. `AudioManager` in `engine_core` owns oddio and implements `BackgroundService`; Lua commands flow through a `thread_local!` queue.

**Tech Stack:** `oddio` (signal graph), `cpal` (audio output, desktop only), `hound` (WAV decoding), `bytemuck` (buffer cast, already in bishop), `mlua` (Lua bindings, already present)

**Spec:** `docs/superpowers/specs/2026-03-23-audio-system-design.md`

---

## File Map

| File | Action | Responsibility |
|------|--------|---------------|
| `engine_core/src/task/mod.rs` | Create | `BackgroundTask<T>`, `BackgroundService` trait |
| `engine_core/src/lib.rs` | Modify | `pub mod task`, prelude re-export |
| `bishop/Cargo.toml` | Modify | `audio`, `audio-cpal` feature flags, `dep:cpal` |
| `bishop/src/audio/mod.rs` | Create | `AudioBackend` trait, `DefaultAudioBackend` type alias |
| `bishop/src/audio/cpal_backend.rs` | Create | `CpalBackend` — cpal stream + bytemuck cast |
| `bishop/src/lib.rs` | Modify | `pub mod audio` (feature-gated), prelude re-export |
| `engine_core/src/constants.rs` | Modify | `AUDIO_FOLDER = "audio"` |
| `engine_core/src/storage/path_utils.rs` | Modify | `audio_folder() -> PathBuf` |
| `engine_core/Cargo.toml` | Modify | `audio` feature, `dep:oddio`, `dep:hound` |
| `engine_core/src/audio/command_queue.rs` | Create | `thread_local!` `AudioCommand` queue, push/drain |
| `engine_core/src/audio/loader.rs` | Create | WAV decode → `oddio::Frames<[f32; 2]>` |
| `engine_core/src/audio/mod.rs` | Create | `AudioManager`, `FadeOut`, `BackgroundService` impl |
| `engine_core/src/lib.rs` | Modify | `pub mod audio` (feature-gated), prelude re-export |
| `game/Cargo.toml` | Modify | `audio` feature enabling `engine_core/audio` |
| `game/src/engine/mod.rs` | Modify | `audio_manager` field, `poll(dt)` call in `frame()` |
| `game/src/engine/engine_builder.rs` | Modify | Pass backend type to `Engine::new` |
| `engine_core/src/scripting/lua_constants.rs` | Modify | Audio Lua string constants |
| `game/src/scripting/modules/audio_module.rs` | Create | `AudioModule` — `engine.audio.*` Lua API |
| `game/src/scripting/modules/mod.rs` | Modify | `pub mod audio_module` |
| `games/Demo/Resources/audio/sfx/jump.wav` | Add | Test audio file for smoke test |
| `games/Demo/Resources/scripts/player.lua` | Modify | Add `engine.audio.play_sfx("sfx/jump")` on jump |

---

## Task 1: Background task contracts

**Files:**
- Create: `engine_core/src/task/mod.rs`
- Modify: `engine_core/src/lib.rs`

- [ ] **Step 1: Create `engine_core/src/task/mod.rs`**

```rust
// engine_core/src/task/mod.rs
use std::sync::mpsc;

/// Runs a closure on a background thread and provides a non-blocking poll
/// to retrieve the result. The main loop must never await — call `poll()`
/// once per frame and handle `Some(result)` when it arrives.
pub struct BackgroundTask<T> {
    receiver: mpsc::Receiver<T>,
}

impl<T: Send + 'static> BackgroundTask<T> {
    /// Spawns `f` on a new thread. Returns immediately.
    pub fn spawn<F: FnOnce() -> T + Send + 'static>(f: F) -> Self {
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(f());
        });
        Self { receiver: rx }
    }

    /// Returns `Some(result)` if the background work is complete, `None` otherwise.
    /// Never blocks.
    pub fn poll(&mut self) -> Option<T> {
        self.receiver.try_recv().ok()
    }
}

/// Contract for a system that runs persistently in the background and must be
/// ticked once per frame. Implementations must never block inside `poll`.
pub trait BackgroundService {
    /// Called once per frame by the game loop. Must never block.
    fn poll(&mut self, dt: f32);
}
```

- [ ] **Step 2: Register the module in `engine_core/src/lib.rs`**

Add after the existing `pub mod` lines:
```rust
pub mod task;
```

Add to the `pub mod prelude` block:
```rust
pub use crate::task::*;
```

- [ ] **Step 3: Verify it builds**

```bash
cargo check -p engine_core
```
Expected: no errors.

- [ ] **Step 4: Commit**

```bash
git add engine_core/src/task/mod.rs engine_core/src/lib.rs
git commit -m "feat(engine_core): add BackgroundTask and BackgroundService contracts"
```

---

## Task 2: AudioBackend trait in bishop

**Files:**
- Modify: `bishop/Cargo.toml`
- Create: `bishop/src/audio/mod.rs`
- Modify: `bishop/src/lib.rs`

- [ ] **Step 1: Add feature flags and cpal dependency to `bishop/Cargo.toml`**

Change the `[features]` block to:
```toml
[features]
default = ["wgpu", "audio-cpal"]
wgpu = ["dep:wgpu", "dep:winit", "dep:pollster", "dep:bytemuck", "dep:image", "dep:fontdue"]
audio = []
audio-cpal = ["audio", "dep:cpal"]
```

Add to `[dependencies]`:
```toml
cpal = { version = "0.15", optional = true }
```

- [ ] **Step 2: Create `bishop/src/audio/mod.rs`**

```rust
// bishop/src/audio/mod.rs

#[cfg(feature = "audio-cpal")]
mod cpal_backend;

#[cfg(feature = "audio-cpal")]
pub use cpal_backend::CpalBackend;

/// Platform audio backend. Starts the audio output stream and calls `render_fn`
/// each buffer to fill samples. Implementors live in bishop; engine_core never
/// depends on the concrete type.
pub trait AudioBackend: Send + 'static {
    /// Starts audio output. `render_fn` is called on the audio thread each buffer.
    /// The render function receives a mutable slice of stereo frames `[[f32; 2]]`.
    fn start<F: FnMut(&mut [[f32; 2]]) + Send + 'static>(render_fn: F) -> Self
    where
        Self: Sized;
}

/// The default audio backend for the current platform, selected by feature flag.
/// Use this in EngineBuilder to avoid hard-coding a backend.
#[cfg(feature = "audio-cpal")]
pub type DefaultAudioBackend = CpalBackend;
```

- [ ] **Step 3: Register the audio module in `bishop/src/lib.rs`**

Add after the `#[cfg(feature = "wgpu")] pub mod wgpu;` line:
```rust
#[cfg(feature = "audio")]
pub mod audio;
```

Add to the `pub mod prelude` block, after the existing `#[cfg(feature = "wgpu")]` lines:
```rust
#[cfg(feature = "audio")]
pub use crate::audio::AudioBackend;

#[cfg(feature = "audio-cpal")]
pub use crate::audio::DefaultAudioBackend;
```

- [ ] **Step 4: Verify it builds**

```bash
cargo check -p bishop
```
Expected: no errors.

- [ ] **Step 5: Commit**

```bash
git add bishop/Cargo.toml bishop/src/audio/mod.rs bishop/src/lib.rs
git commit -m "feat(bishop): add AudioBackend trait and audio feature flags"
```

---

## Task 3: CpalBackend

**Files:**
- Create: `bishop/src/audio/cpal_backend.rs`

cpal delivers samples as a flat interleaved `&mut [f32]` buffer. `CpalBackend` casts it to `&mut [[f32; 2]]` using `bytemuck::cast_slice_mut` (bytemuck is already a dependency under the `wgpu` feature — add it to `audio-cpal` too in `bishop/Cargo.toml`).

- [ ] **Step 1: Add bytemuck to the `audio-cpal` feature in `bishop/Cargo.toml`**

Change:
```toml
audio-cpal = ["audio", "dep:cpal"]
```
to:
```toml
audio-cpal = ["audio", "dep:cpal", "dep:bytemuck"]
```

- [ ] **Step 2: Create `bishop/src/audio/cpal_backend.rs`**

No `.expect()` or `.unwrap()` — on failure (no device, unsupported format) log the error and return a shell with no stream. Audio is simply silent rather than crashing.

```rust
// bishop/src/audio/cpal_backend.rs
use crate::audio::AudioBackend;
use bytemuck::cast_slice_mut;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream, StreamConfig};

/// Desktop audio backend using cpal. Holds the output stream alive for the
/// lifetime of the AudioManager. Dropping this stops all audio.
/// `_stream` is `None` when no audio device is available (headless / CI).
pub struct CpalBackend {
    /// Keeps the cpal stream alive. None means audio is disabled (no device found).
    _stream: Option<Stream>,
}

// Stream is not Send by default on some platforms; asserted safe here because
// _stream is never accessed after start() returns — it is only kept alive by drop.
unsafe impl Send for CpalBackend {}

impl AudioBackend for CpalBackend {
    fn start<F: FnMut(&mut [[f32; 2]]) + Send + 'static>(mut render_fn: F) -> Self {
        let host = cpal::default_host();

        let device = match host.default_output_device() {
            Some(d) => d,
            None => {
                log::error!("no audio output device found — audio disabled");
                return Self { _stream: None };
            }
        };

        let supported = match device.default_output_config() {
            Ok(c) => c,
            Err(e) => {
                log::error!("no default audio output config: {e} — audio disabled");
                return Self { _stream: None };
            }
        };

        if supported.sample_format() != SampleFormat::F32 {
            log::error!(
                "unsupported sample format {:?} — audio disabled",
                supported.sample_format()
            );
            return Self { _stream: None };
        }

        let config = StreamConfig {
            channels: 2,
            sample_rate: supported.sample_rate(),
            buffer_size: cpal::BufferSize::Default,
        };

        let stream = match device.build_output_stream(
            &config,
            move |data: &mut [f32], _| {
                // Cast flat interleaved f32 buffer to stereo frames for oddio.
                // Safety: [f32; 2] has the same layout as two consecutive f32s.
                let frames: &mut [[f32; 2]] = cast_slice_mut(data);
                render_fn(frames);
            },
            |err| log::error!("audio stream error: {err}"),
            None,
        ) {
            Ok(s) => s,
            Err(e) => {
                log::error!("failed to build audio output stream: {e} — audio disabled");
                return Self { _stream: None };
            }
        };

        if let Err(e) = stream.play() {
            log::error!("failed to start audio stream: {e} — audio disabled");
            return Self { _stream: None };
        }

        Self { _stream: Some(stream) }
    }
}
```

- [ ] **Step 3: Verify it builds**

```bash
cargo check -p bishop
```
Expected: no errors.

- [ ] **Step 4: Commit**

```bash
git add bishop/Cargo.toml bishop/src/audio/cpal_backend.rs
git commit -m "feat(bishop): add CpalBackend — cpal output stream with stereo frame cast"
```

---

## Task 4: Audio folder constant and path helper

**Files:**
- Modify: `engine_core/src/constants.rs`
- Modify: `engine_core/src/storage/path_utils.rs`

- [ ] **Step 1: Add `AUDIO_FOLDER` to `engine_core/src/constants.rs`**

Add after the `MENUS_FOLDER` line:
```rust
/// Name of the audio folder.
pub const AUDIO_FOLDER: &str = "audio";
```

- [ ] **Step 2: Add `audio_folder()` to `engine_core/src/storage/path_utils.rs`**

Add after the `menus_folder()` function:
```rust
/// Path to the audio folder inside the resources folder (Editor/Game).
pub fn audio_folder() -> PathBuf {
    resources_folder_current().join(AUDIO_FOLDER)
}
```

- [ ] **Step 3: Verify it builds**

```bash
cargo check -p engine_core
```
Expected: no errors.

- [ ] **Step 4: Commit**

```bash
git add engine_core/src/constants.rs engine_core/src/storage/path_utils.rs
git commit -m "feat(engine_core): add AUDIO_FOLDER constant and audio_folder() path helper"
```

---

## Task 5: engine_core audio feature + AudioCommand queue

**Files:**
- Modify: `engine_core/Cargo.toml`
- Create: `engine_core/src/audio/command_queue.rs`
- Create: `engine_core/src/audio/mod.rs` (stub)
- Modify: `engine_core/src/lib.rs`

- [ ] **Step 1: Add audio feature and dependencies to `engine_core/Cargo.toml`**

Change the `[features]` block to:
```toml
[features]
default = ["wgpu"]
wgpu = ["bishop/wgpu", "widgets/wgpu"]
editor = []
audio = ["bishop/audio", "dep:oddio", "dep:hound"]
```

Add to `[dependencies]`:
```toml
oddio = { version = "0.6", optional = true }
hound = { version = "3.5", optional = true }
```

> Note: Verify the exact `oddio` version at https://crates.io/crates/oddio before pinning.

- [ ] **Step 2: Create `engine_core/src/audio/command_queue.rs`**

```rust
// engine_core/src/audio/command_queue.rs
use std::cell::RefCell;

/// Commands that Lua scripts can issue to the audio system.
/// Queued on the main thread, drained by `AudioManager::poll` each frame.
pub enum AudioCommand {
    PlayMusic(String),
    StopMusic,
    FadeMusic(f32),
    PlaySfx(String),
    Preload(String),
    SetMasterVolume(f32),
    SetMusicVolume(f32),
    SetSfxVolume(f32),
}

thread_local! {
    static AUDIO_COMMANDS: RefCell<Vec<AudioCommand>> = const { RefCell::new(Vec::new()) };
}

/// Push a command onto the per-frame audio queue. Safe to call from Lua closures.
pub fn push_audio_command(cmd: AudioCommand) {
    AUDIO_COMMANDS.with(|q| q.borrow_mut().push(cmd));
}

/// Drain all queued commands. Called once per frame by `AudioManager::poll`.
pub fn drain_audio_commands() -> Vec<AudioCommand> {
    AUDIO_COMMANDS.with(|q| {
        let mut v = q.borrow_mut();
        std::mem::take(&mut *v)
    })
}
```

- [ ] **Step 3: Create stub `engine_core/src/audio/mod.rs`**

```rust
// engine_core/src/audio/mod.rs
pub mod command_queue;
pub use command_queue::{AudioCommand, push_audio_command, drain_audio_commands};
```

- [ ] **Step 4: Register the audio module in `engine_core/src/lib.rs`**

Add after the existing `pub mod` lines:
```rust
#[cfg(feature = "audio")]
pub mod audio;
```

Add to the `pub mod prelude` block — export only the public API, not the internal drain function:
```rust
#[cfg(feature = "audio")]
pub use crate::audio::{AudioCommand, AudioManager, push_audio_command};
```

- [ ] **Step 5: Verify it builds**

```bash
cargo check -p engine_core --features audio
```
Expected: no errors.

- [ ] **Step 6: Commit**

```bash
git add engine_core/Cargo.toml engine_core/src/audio/command_queue.rs engine_core/src/audio/mod.rs engine_core/src/lib.rs
git commit -m "feat(engine_core): add audio feature, AudioCommand queue"
```

---

## Task 6: WAV loader

**Files:**
- Create: `engine_core/src/audio/loader.rs`
- Modify: `engine_core/src/audio/mod.rs`

- [ ] **Step 1: Create `engine_core/src/audio/loader.rs`**

Decodes a WAV file by ID to stereo f32 PCM frames. The string ID `"sfx/jump"` resolves to `audio_folder()/sfx/jump.wav`. The cache key is always the raw ID.

```rust
// engine_core/src/audio/loader.rs
use crate::storage::path_utils::audio_folder;
use oddio::Frames;
use std::sync::Arc;

/// Loads a WAV file by sound ID and decodes it to stereo f32 PCM frames
/// suitable for oddio playback.
///
/// The ID is a path relative to the `audio/` folder without extension,
/// e.g. `"sfx/jump"` resolves to `Resources/audio/sfx/jump.wav`.
pub fn load_wav(id: &str) -> Result<Arc<Frames<[f32; 2]>>, String> {
    let path = audio_folder().join(id).with_extension("wav");
    let mut reader = hound::WavReader::open(&path)
        .map_err(|e| format!("failed to open {}: {e}", path.display()))?;

    let spec = reader.spec();
    let sample_rate = spec.sample_rate;
    let channels = spec.channels as usize;

    // Decode all samples to f32, regardless of source bit depth.
    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => reader
            .samples::<f32>()
            .map(|s| s.map_err(|e| e.to_string()))
            .collect::<Result<_, _>>()?,
        hound::SampleFormat::Int => {
            let max = (1i64 << (spec.bits_per_sample - 1)) as f32;
            reader
                .samples::<i32>()
                .map(|s| s.map(|v| v as f32 / max).map_err(|e| e.to_string()))
                .collect::<Result<_, _>>()?
        }
    };

    // Interleave or mix down to stereo [f32; 2] frames.
    let frames: Vec<[f32; 2]> = match channels {
        1 => samples.iter().map(|&s| [s, s]).collect(),
        2 => samples.chunks_exact(2).map(|c| [c[0], c[1]]).collect(),
        n => {
            return Err(format!(
                "unsupported channel count {n} in {}",
                path.display()
            ))
        }
    };

    Ok(Arc::new(Frames::from_slice(sample_rate, &frames)))
}
```

- [ ] **Step 2: Register loader in `engine_core/src/audio/mod.rs`**

```rust
// engine_core/src/audio/mod.rs
pub mod command_queue;
pub mod loader;

pub use command_queue::{AudioCommand, drain_audio_commands, push_audio_command};
pub use loader::load_wav;
```

- [ ] **Step 3: Verify it builds**

```bash
cargo check -p engine_core --features audio
```
Expected: no errors.

- [ ] **Step 4: Commit**

```bash
git add engine_core/src/audio/loader.rs engine_core/src/audio/mod.rs
git commit -m "feat(engine_core): add WAV loader — decodes to oddio stereo frames"
```

---

## Task 7: AudioManager

**Files:**
- Modify: `engine_core/src/audio/mod.rs` (add `AudioManager`, `FadeOut`, `BackgroundService` impl)

This is the largest task. oddio's signal graph:
- `oddio::Scene` is the root mixer, split into `(Handle<Scene>, impl Signal)`.
- Add `Gain<Mixer>` signals into the scene for music and sfx groups.
- `Handle<Gain<...>>` exposes `set_gain(gain: f32)`.
- Music plays as `Cycle<FramesSignal>` wrapped in `Stop<...>` so it can be stopped.
- SFX plays as `FramesSignal` fire-and-forget via the sfx mixer.
- Fades are driven by `FadeOut` in `poll()`.

> Note: Verify the exact `oddio` API against the crate documentation for the pinned version. The signal types and method names below match the public oddio 0.6 API — adjust if the version differs.

- [ ] **Step 1: Pin the oddio version and read its API docs before writing any code**

Before writing `AudioManager`, run:
```bash
cargo add oddio -p engine_core
```
Then open the docs:
```bash
cargo doc -p engine_core --features audio --open
```
Read the oddio docs specifically for: `Mixer`, `Gain`, `Stop`, `Cycle`, `FramesSignal`, `Frames`, and `split`. The code below uses the correct concepts but the exact method names must be verified. `oddio::Scene` is for **spatial 3D audio** — the root for stereo mixing is `oddio::Mixer<[f32; 2]>`.

- [ ] **Step 2: Replace the stub body of `engine_core/src/audio/mod.rs` with the full implementation**

The signal graph:
- Root: `Mixer<[f32; 2]>` — split into `(Handle<Mixer>, impl Signal)` for the backend
- Music group: `Gain<Mixer<[f32; 2]>>` played into the root mixer
- SFX group: `Gain<Mixer<[f32; 2]>>` played into the root mixer
- Music track: `Stop<Cycle<FramesSignal<[f32; 2]>>>` — `Cycle` makes it loop, `Stop` makes it stoppable
- SFX: `FramesSignal<[f32; 2]>` fire-and-forget

```rust
// engine_core/src/audio/mod.rs
pub mod command_queue;
pub mod loader;

pub use command_queue::{AudioCommand, drain_audio_commands, push_audio_command};
pub use loader::load_wav;

use crate::task::BackgroundService;
use bishop::audio::AudioBackend;
use oddio::{Cycle, Frames, FramesSignal, Gain, Handle, Mixer, Stop};
use std::collections::HashMap;
use std::sync::Arc;

/// Time-remaining state for a music fade-out.
struct FadeOut {
    remaining: f32,
    duration: f32,
}

/// Manages audio playback for the game. Implements [`BackgroundService`] — call
/// `poll(dt)` once per frame. Never call blocking audio operations on the
/// game thread; issue [`AudioCommand`]s instead.
pub struct AudioManager {
    /// Keeps the audio backend stream alive. Dropping this field stops all audio.
    _keep_alive: Box<dyn Send + 'static>,
    /// Root stereo mixer. The backend renders this each buffer.
    root: Handle<Mixer<[f32; 2]>>,
    /// Music group with independent gain control.
    music_mixer: Handle<Gain<Mixer<[f32; 2]>>>,
    /// SFX group with independent gain control.
    sfx_mixer: Handle<Gain<Mixer<[f32; 2]>>>,
    /// Handle to the active music track (looping, stoppable). None if no music playing.
    active_music: Option<Handle<Stop<Cycle<FramesSignal<[f32; 2]>>>>>,
    active_fade: Option<FadeOut>,
    /// Sound cache keyed by raw ID string (e.g. "sfx/jump"). Never evicted.
    sound_cache: HashMap<String, Arc<Frames<[f32; 2]>>>,
    master_volume: f32,
    music_volume: f32,
    sfx_volume: f32,
}

impl AudioManager {
    /// Constructs the audio manager, starts the backend, and builds the signal graph.
    /// `B` is the platform audio backend (e.g. `bishop::audio::DefaultAudioBackend`).
    ///
    /// **IMPORTANT:** The oddio API calls in this function (split, play, etc.) must be
    /// verified against the pinned oddio version before running. Consult `cargo doc`.
    pub fn new<B: AudioBackend>() -> Self {
        // Root stereo mixer. The backend renders its output each buffer.
        let (root_handle, root_renderer) = oddio::split(Mixer::new());

        // Music group: Gain<Mixer> played into the root.
        // Exact API: verify `root_handle.play(signal)` vs `root_handle.control().play(signal)`
        let music_group = Gain::new(Mixer::new());
        let music_mixer_handle = root_handle.play(music_group);   // adjust if API differs

        // SFX group: Gain<Mixer> played into the root.
        let sfx_group = Gain::new(Mixer::new());
        let sfx_mixer_handle = root_handle.play(sfx_group);       // adjust if API differs

        // Start the backend; it renders the root mixer each buffer.
        let _keep_alive: Box<dyn Send + 'static> = Box::new(
            B::start(move |frames| root_renderer.render(frames))  // adjust method name if needed
        );

        Self {
            _keep_alive,
            root: root_handle,
            music_mixer: music_mixer_handle,
            sfx_mixer: sfx_mixer_handle,
            active_music: None,
            active_fade: None,
            sound_cache: HashMap::new(),
            master_volume: 1.0,
            music_volume: 1.0,
            sfx_volume: 1.0,
        }
    }

    /// Pre-loads a sound file into the cache. Use during `Game::initialize` for
    /// SFX that must never stutter on their first play.
    pub fn preload(&mut self, id: &str) {
        if !self.sound_cache.contains_key(id) {
            match load_wav(id) {
                Ok(frames) => { self.sound_cache.insert(id.to_string(), frames); }
                Err(e) => log::error!("audio preload failed for '{id}': {e}"),
            }
        }
    }

    /// Plays a sound effect fire-and-forget on the sfx mixer.
    pub fn play_sfx(&mut self, id: &str) {
        if let Some(frames) = self.cached_or_load(id) {
            let signal = FramesSignal::from(frames);
            // Play into the inner Mixer of sfx_mixer.
            // Adjust the control/play pattern to match the oddio version.
            self.sfx_mixer.control::<Mixer<[f32; 2]>, _>().play(signal);
        }
    }

    /// Starts looping music, stopping any currently playing track first.
    /// Music loops via `Cycle` and can be stopped or faded.
    pub fn play_music(&mut self, id: &str) {
        self.stop_music();
        if let Some(frames) = self.cached_or_load(id) {
            // Cycle makes it loop; Stop makes it stoppable from the game thread.
            let signal = Stop::new(Cycle::new(FramesSignal::from(frames)));
            let handle = self.music_mixer.control::<Mixer<[f32; 2]>, _>().play(signal);
            self.active_music = Some(handle);
            self.active_fade = None;
        }
    }

    /// Stops music immediately.
    pub fn stop_music(&mut self) {
        if let Some(ref mut handle) = self.active_music {
            handle.control::<Stop<_>, _>().stop();
        }
        self.active_music = None;
        self.active_fade = None;
    }

    /// Fades music out over `duration` seconds, then stops it.
    pub fn fade_music(&mut self, duration: f32) {
        if self.active_music.is_some() {
            self.active_fade = Some(FadeOut { remaining: duration, duration });
        }
    }

    /// Sets master volume, clamped to [0.0, 1.0]. Updates all group gains.
    pub fn set_master_volume(&mut self, v: f32) {
        self.master_volume = v.clamp(0.0, 1.0);
        self.apply_music_gain();
        self.apply_sfx_gain();
    }

    /// Sets music group volume, clamped to [0.0, 1.0].
    pub fn set_music_volume(&mut self, v: f32) {
        self.music_volume = v.clamp(0.0, 1.0);
        self.apply_music_gain();
    }

    /// Sets SFX group volume, clamped to [0.0, 1.0].
    pub fn set_sfx_volume(&mut self, v: f32) {
        self.sfx_volume = v.clamp(0.0, 1.0);
        self.apply_sfx_gain();
    }

    fn apply_music_gain(&mut self) {
        let gain = self.music_volume * self.master_volume;
        self.music_mixer.control::<Gain<_>, _>().set_gain(gain);
    }

    fn apply_sfx_gain(&mut self) {
        let gain = self.sfx_volume * self.master_volume;
        self.sfx_mixer.control::<Gain<_>, _>().set_gain(gain);
    }

    fn cached_or_load(&mut self, id: &str) -> Option<Arc<Frames<[f32; 2]>>> {
        if !self.sound_cache.contains_key(id) {
            match load_wav(id) {
                Ok(frames) => { self.sound_cache.insert(id.to_string(), frames); }
                Err(e) => {
                    log::error!("audio load failed for '{id}': {e}");
                    return None;
                }
            }
        }
        self.sound_cache.get(id).cloned()
    }
}

impl BackgroundService for AudioManager {
    fn poll(&mut self, dt: f32) {
        // 1. Advance fade and update gain before processing commands.
        if let Some(ref mut fade) = self.active_fade {
            fade.remaining -= dt;
            if fade.remaining <= 0.0 {
                self.stop_music();
            } else {
                let t = fade.remaining / fade.duration;
                let gain = self.music_volume * self.master_volume * t;
                self.music_mixer.control::<Gain<_>, _>().set_gain(gain);
            }
        }

        // 2. Drain and execute audio commands from Lua.
        for cmd in drain_audio_commands() {
            match cmd {
                AudioCommand::PlayMusic(id) => self.play_music(&id),
                AudioCommand::StopMusic => self.stop_music(),
                AudioCommand::FadeMusic(dur) => self.fade_music(dur),
                AudioCommand::PlaySfx(id) => self.play_sfx(&id),
                AudioCommand::Preload(id) => self.preload(&id),
                AudioCommand::SetMasterVolume(v) => self.set_master_volume(v),
                AudioCommand::SetMusicVolume(v) => self.set_music_volume(v),
                AudioCommand::SetSfxVolume(v) => self.set_sfx_volume(v),
            }
        }
    }
}
```

- [ ] **Step 2: Verify it builds**

```bash
cargo check -p engine_core --features audio
```
Expected: no errors. Fix any oddio API mismatches at this point.

- [ ] **Step 3: Commit**

```bash
git add engine_core/src/audio/mod.rs
git commit -m "feat(engine_core): implement AudioManager with volume groups and fade"
```

---

## Task 8: Wire AudioManager into the Engine

**Files:**
- Modify: `game/Cargo.toml`
- Modify: `game/src/engine/mod.rs`
- Modify: `game/src/engine/engine_builder.rs`

- [ ] **Step 1: Enable the audio feature in `game/Cargo.toml`**

Change the `[features]` block to:
```toml
[features]
default = ["wgpu", "audio"]
wgpu = ["bishop/wgpu", "engine_core/wgpu", "widgets/wgpu"]
audio = ["engine_core/audio", "bishop/audio-cpal"]
```

- [ ] **Step 2: Add `audio_manager` field to the `Engine` struct in `game/src/engine/mod.rs`**

In the `Engine` struct, add after `pub smoothed_dt`:
```rust
/// Background audio service, polled once per frame.
#[cfg(feature = "audio")]
pub audio_manager: engine_core::audio::AudioManager,
```

In `Engine::new()`, add a type parameter and construct AudioManager. Change the signature to:
```rust
pub fn new<B: bishop::audio::AudioBackend>(
    game_instance: Rc<RefCell<GameInstance>>,
    ctx: PlatformContext,
    lua: Lua,
    camera_manager: CameraManager,
    grid_size: f32,
    is_playtest: bool,
) -> Self {
```

Add to the `Self { ... }` constructor body:
```rust
#[cfg(feature = "audio")]
audio_manager: engine_core::audio::AudioManager::new::<B>(),
```

In `BishopApp for Engine`, add `audio_manager.poll(raw_dt)` at the top of `frame()`, right after `let raw_dt = ctx.borrow().get_frame_time();`:
```rust
#[cfg(feature = "audio")]
self.audio_manager.poll(raw_dt);
```

- [ ] **Step 3: Update `EngineBuilder::assemble()` to pass the backend type**

Change the call in `engine_builder.rs` from:
```rust
Engine::new(game_instance, ctx, self.lua, self.camera_manager, grid_size, is_playtest)
```
to:
```rust
Engine::new::<bishop::prelude::DefaultAudioBackend>(
    game_instance,
    ctx,
    self.lua,
    self.camera_manager,
    grid_size,
    is_playtest,
)
```

- [ ] **Step 4: Build the full game**

```bash
cargo build -p game
```
Expected: clean build. No audio plays yet — the system is wired but no commands issued.

- [ ] **Step 5: Commit**

```bash
git add game/Cargo.toml game/src/engine/mod.rs game/src/engine/engine_builder.rs
git commit -m "feat(game): wire AudioManager into Engine, poll each frame"
```

---

## Task 9: Lua constants and AudioModule

**Files:**
- Modify: `engine_core/src/scripting/lua_constants.rs`
- Create: `game/src/scripting/modules/audio_module.rs`
- Modify: `game/src/scripting/modules/mod.rs`

- [ ] **Step 1: Add audio Lua constants to `engine_core/src/scripting/lua_constants.rs`**

Add at the end of the file:
```rust
// Audio module
pub const LUA_AUDIO: &str = "audio";
pub const AUDIO_FILE: &str = "audio.lua";
pub const AUDIO_PLAY_MUSIC: &str = "play_music";
pub const AUDIO_STOP_MUSIC: &str = "stop_music";
pub const AUDIO_FADE_MUSIC: &str = "fade_music";
pub const AUDIO_PLAY_SFX: &str = "play_sfx";
pub const AUDIO_PRELOAD: &str = "preload";
pub const AUDIO_SET_MASTER_VOLUME: &str = "set_master_volume";
pub const AUDIO_SET_MUSIC_VOLUME: &str = "set_music_volume";
pub const AUDIO_SET_SFX_VOLUME: &str = "set_sfx_volume";
```

- [ ] **Step 2: Create `game/src/scripting/modules/audio_module.rs`**

```rust
// game/src/scripting/modules/audio_module.rs
use engine_core::audio::{push_audio_command, AudioCommand};
use engine_core::register_lua_api;
use engine_core::register_lua_module;
use engine_core::scripting::modules::lua_module::*;
use engine_core::scripting::lua_constants::*;
use mlua::prelude::LuaResult;
use mlua::Table;
use mlua::Lua;

/// Lua module that exposes the audio system API under `engine.audio`.
#[derive(Default)]
pub struct AudioModule;
register_lua_module!(AudioModule);

impl LuaModule for AudioModule {
    fn register(&self, lua: &Lua) -> LuaResult<()> {
        let engine_tbl: Table = lua.globals().get(ENGINE)?;
        let audio_tbl = lua.create_table()?;

        audio_tbl.set(AUDIO_PLAY_MUSIC, lua.create_function(|_, id: String| {
            push_audio_command(AudioCommand::PlayMusic(id));
            Ok(())
        })?)?;

        audio_tbl.set(AUDIO_STOP_MUSIC, lua.create_function(|_, ()| {
            push_audio_command(AudioCommand::StopMusic);
            Ok(())
        })?)?;

        audio_tbl.set(AUDIO_FADE_MUSIC, lua.create_function(|_, duration: f32| {
            push_audio_command(AudioCommand::FadeMusic(duration));
            Ok(())
        })?)?;

        audio_tbl.set(AUDIO_PLAY_SFX, lua.create_function(|_, id: String| {
            push_audio_command(AudioCommand::PlaySfx(id));
            Ok(())
        })?)?;

        audio_tbl.set(AUDIO_PRELOAD, lua.create_function(|_, id: String| {
            push_audio_command(AudioCommand::Preload(id));
            Ok(())
        })?)?;

        audio_tbl.set(AUDIO_SET_MASTER_VOLUME, lua.create_function(|_, v: f32| {
            push_audio_command(AudioCommand::SetMasterVolume(v));
            Ok(())
        })?)?;

        audio_tbl.set(AUDIO_SET_MUSIC_VOLUME, lua.create_function(|_, v: f32| {
            push_audio_command(AudioCommand::SetMusicVolume(v));
            Ok(())
        })?)?;

        audio_tbl.set(AUDIO_SET_SFX_VOLUME, lua.create_function(|_, v: f32| {
            push_audio_command(AudioCommand::SetSfxVolume(v));
            Ok(())
        })?)?;

        engine_tbl.set(LUA_AUDIO, audio_tbl)?;
        Ok(())
    }
}

register_lua_api!(AudioModule, AUDIO_FILE);

impl LuaApi for AudioModule {
    fn emit_api(&self, out: &mut LuaApiWriter) {
        out.line("--- Audio system module");
        out.line("---@class AudioApi");
        out.line("engine.audio = {}");
        out.line("");
        out.line("--- Plays music by ID, looping until stopped. Stops any current track.");
        out.line("---@param id string Path relative to Resources/audio/ without extension");
        out.line("function engine.audio.play_music(id) end");
        out.line("");
        out.line("--- Stops music immediately.");
        out.line("function engine.audio.stop_music() end");
        out.line("");
        out.line("--- Fades music out over duration seconds, then stops.");
        out.line("---@param duration number Fade duration in seconds");
        out.line("function engine.audio.fade_music(duration) end");
        out.line("");
        out.line("--- Plays a sound effect fire-and-forget.");
        out.line("---@param id string Path relative to Resources/audio/ without extension");
        out.line("function engine.audio.play_sfx(id) end");
        out.line("");
        out.line("--- Pre-loads a sound into the cache to prevent stutter on first play.");
        out.line("---@param id string Path relative to Resources/audio/ without extension");
        out.line("function engine.audio.preload(id) end");
        out.line("");
        out.line("--- Sets master volume (0.0–1.0).");
        out.line("---@param volume number");
        out.line("function engine.audio.set_master_volume(volume) end");
        out.line("");
        out.line("--- Sets music group volume (0.0–1.0).");
        out.line("---@param volume number");
        out.line("function engine.audio.set_music_volume(volume) end");
        out.line("");
        out.line("--- Sets SFX group volume (0.0–1.0).");
        out.line("---@param volume number");
        out.line("function engine.audio.set_sfx_volume(volume) end");
    }
}
```

- [ ] **Step 3: Register the module in `game/src/scripting/modules/mod.rs`**

Add:
```rust
#[cfg(feature = "audio")]
pub mod audio_module;
```

> **Auto-registration note:** No manual wiring is needed. `register_lua_module!(AudioModule)` submits the module to the `inventory` crate at link time (`inventory::submit!`). The engine startup code in `engine_core/src/scripting/` uses `inventory::iter::<LuaModuleRegistry>` to collect and call `register()` on all submitted modules. Just declaring the module with `register_lua_module!` is sufficient — as long as the `pub mod audio_module` line above causes it to be compiled in.

- [ ] **Step 4: Build the full game**

```bash
cargo build -p game
```
Expected: clean build.

- [ ] **Step 5: Commit**

```bash
git add engine_core/src/scripting/lua_constants.rs game/src/scripting/modules/audio_module.rs game/src/scripting/modules/mod.rs
git commit -m "feat(game): add AudioModule — engine.audio Lua API"
```

---

## Task 10: End-to-end smoke test

**Files:**
- Add: `games/Demo/Resources/audio/sfx/jump.wav`
- Modify: `games/Demo/Resources/scripts/player.lua`

- [ ] **Step 1: Create the audio folder and add a test WAV**

```bash
mkdir -p games/Demo/Resources/audio/sfx
```

Obtain any short stereo WAV file (44100 Hz recommended) and copy it to `games/Demo/Resources/audio/sfx/jump.wav`. A free source: freesound.org, or generate a 0.1s 440 Hz sine wave with Audacity.

- [ ] **Step 2: Add `play_sfx` to the jump in `player.lua`**

In `games/Demo/Resources/scripts/player.lua`, change:
```lua
if engine.input.is_down(input.Space) and is_grounded then
    new_vel.y = -self.public.jump_speed
end
```
to:
```lua
if engine.input.is_down(input.Space) and is_grounded then
    new_vel.y = -self.public.jump_speed
    engine.audio.play_sfx("sfx/jump")
end
```

- [ ] **Step 3: Run the game and test**

```bash
cargo run -p game
```

Jump with Space. Expected: a sound plays on each jump. Check the terminal for any `audio load failed` or `audio stream error` log lines.

- [ ] **Step 4: Commit**

```bash
git add games/Demo/Resources/audio/sfx/jump.wav games/Demo/Resources/scripts/player.lua
git commit -m "test: add jump SFX smoke test for audio system"
```
