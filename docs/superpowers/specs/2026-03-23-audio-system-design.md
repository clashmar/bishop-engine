# Audio System Design

**Date:** 2026-03-23
**Branch:** 2026-03-19-AUDIO-SYSTEM
**Status:** Approved

---

## Overview

This document specifies the audio system and background task infrastructure for the bishop engine. Audio is the first system requiring a dedicated thread; the contracts established here (fire-and-poll) are the model all future background systems (save/load, asset streaming, pathfinding) must follow.

---

## 1. Background Task Contracts

Two types live in a new `engine_core/src/task/` module, re-exported via `engine_core::prelude`.

### `BackgroundTask<T>` — one-shot work

Wraps a `std::thread` and `std::sync::mpsc` channel. The caller spawns work and polls for the result each frame. Never blocks.

```rust
pub struct BackgroundTask<T> {
    receiver: mpsc::Receiver<T>,
}

impl<T: Send + 'static> BackgroundTask<T> {
    pub fn spawn<F: FnOnce() -> T + Send + 'static>(f: F) -> Self {
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || { let _ = tx.send(f()); });
        Self { receiver: rx }
    }

    pub fn poll(&mut self) -> Option<T> {
        self.receiver.try_recv().ok()
    }
}
```

**Use for:** save/load, asset streaming, pathfinding. Caller stores `Option<BackgroundTask<T>>`, calls `.poll()` each frame, handles the result when `Some`.

### `BackgroundService` trait — persistent services

```rust
pub trait BackgroundService {
    /// Called once per frame by the game loop. Must never block.
    fn poll(&mut self, dt: f32);
}
```

**Use for:** `AudioManager`, future network layer. The game loop calls `poll(dt)` each frame unconditionally. `dt` is the same frame delta passed through the rest of the update — fade timers and other time-based logic use it directly.

### Rationale

These two contracts enforce the fire-and-poll pattern at the type level. `BackgroundTask` cannot block — `try_recv` either returns a value or returns immediately. `BackgroundService::poll` is documented as non-blocking by contract. Neither uses tokio, preserving console portability.

---

## 2. Audio Backend (bishop)

Platform-specific audio lives in `bishop/src/audio/`, alongside the existing wgpu backend. This keeps all platform code in one crate.

### `AudioBackend` trait

Bishop does not depend on oddio. The backend contract is a render callback — every audio API on every platform is fundamentally "fill this buffer":

```rust
// bishop/src/audio/mod.rs
pub trait AudioBackend: Send + 'static {
    /// Starts audio output. `render_fn` is called each buffer to fill samples.
    fn start<F: FnMut(&mut [[f32; 2]]) + Send + 'static>(render_fn: F) -> Self
    where Self: Sized;
}
```

### `CpalBackend`

Desktop default implementation in `bishop/src/audio/cpal_backend.rs`, compiled only under the `audio-cpal` feature flag (default on desktop). Uses cpal to create an output stream that calls `render_fn` each audio buffer.

### Feature flags in bishop

```toml
[features]
default = ["wgpu", "audio-cpal"]
wgpu = [...]
audio-cpal = ["dep:cpal"]
```

Future console backends (`nx_backend.rs`, `ps5_backend.rs`, etc.) sit alongside `cpal_backend.rs` under their own feature flags.

### Separation of concerns

| Crate | Knows about |
|-------|-------------|
| bishop | Audio hardware (cpal, console SDKs) |
| engine_core | oddio signal graph, game audio logic |

---

## 3. Audio System (engine_core)

Lives in `engine_core/src/audio/`. Requires the `audio` feature flag on `engine_core`.

### `AudioManager`

Implements `BackgroundService`. Not generic — the backend type is erased at construction:

```rust
pub struct AudioManager {
    /// Keeps the audio backend stream alive. Dropping this field silently stops all audio.
    _keep_alive: Box<dyn Send + 'static>,
    scene: oddio::Handle<oddio::Scene<[f32; 2]>>,
    music_mixer: oddio::Handle<oddio::Gain<oddio::Mixer<[f32; 2]>>>,
    sfx_mixer:   oddio::Handle<oddio::Gain<oddio::Mixer<[f32; 2]>>>,
    active_music: Option<oddio::Handle<oddio::Stop<oddio::Cycle<oddio::FramesSignal<[f32; 2]>>>>>,
    active_fade:  Option<FadeOut>,
    sound_cache:  HashMap<String, Arc<oddio::Frames<[f32; 2], f32>>>,
    master_volume: f32,
    music_volume:  f32,
    sfx_volume:    f32,
}

impl AudioManager {
    pub fn new<B: AudioBackend>() -> Self {
        let (scene_handle, scene_renderer) = oddio::split(oddio::Scene::new());
        let _keep_alive: Box<dyn Send + 'static> = Box::new(
            B::start(move |frames| scene_renderer.render(frames))
        );
        // ... build music_mixer and sfx_mixer into scene
    }
}
```

### Volume groups

Music and SFX are separate `Gain`-wrapped `Mixer` signals played into the top-level scene. Setting volume multiplies the group volume by master:

```
scene
  └── music_mixer: Gain<Mixer>   ← music_volume * master_volume
  └── sfx_mixer:   Gain<Mixer>   ← sfx_volume * master_volume
```

A master volume change updates both mixer gains. Per-sound gain is not exposed (not needed yet).

### Fades

`FadeOut { handle, remaining, duration }` — no threads, no timers. `poll()` decrements `remaining` by `dt` and updates the music mixer gain proportionally each frame. When `remaining` reaches zero, the music is stopped.

### `poll()` implementation

```rust
impl BackgroundService for AudioManager {
    fn poll(&mut self, dt: f32) {
        // 1. Drain AudioCommands from the global command queue
        // 2. Advance active fade by dt and update gain
    }
}
```

---

## 4. Asset Loading

### File layout

```
Resources/
    audio/
        sfx/
            jump.wav
            footstep.wav
        music/
            theme_main.wav
```

A new constant `AUDIO_FOLDER = "audio"` is added to `engine_core/src/constants.rs`. A new `audio_folder()` function is added to `path_utils.rs`, following the same pattern as `assets_folder()` and `scripts_folder()`.

### ID resolution

String ID → file path: `"sfx/jump"` → `audio_folder().join("sfx/jump.wav")`. The extension (`.wav`) is appended automatically. Subdirectory depth is unrestricted.

### In-memory cache

`AudioManager` owns `sound_cache: HashMap<String, Arc<oddio::Frames<[f32; 2], f32>>>`. `Arc` allows multiple simultaneous plays of the same sound to share decoded data.

- **`preload(name)`** — loads and caches a file explicitly. Called during `Game::initialize` for SFX that must never stutter on first play.
- **`play_sfx(name)`** — checks cache, loads on miss, plays immediately.
- **Music** — loaded on demand. One track at a time; loaded before the scene that needs it.
- **No eviction** — games are small. LRU can be added later without changing the API.
- **Cache key** — always the raw sound ID string (e.g. `"sfx/jump"`), never the resolved file path. Both `preload` and `play_sfx` must use the same key so a pre-loaded sound is always found on the cache hit path.

### Format

WAV only initially, decoded to `oddio::Frames<[f32; 2], f32>` (stereo f32 PCM) at load time. OGG and other formats added later without API changes.

---

## 5. Lua API

Registered in `AudioLuaModule` under the `engine.audio` namespace, consistent with `engine.input`, `engine.menu`, etc.

```lua
engine.audio.play_music("music/theme_main")   -- loops until stopped or faded
engine.audio.stop_music()                      -- immediate cut
engine.audio.fade_music(2.0)                   -- fade out over N seconds
engine.audio.play_sfx("sfx/jump")              -- fire-and-forget
engine.audio.preload("sfx/footstep")           -- warm the cache explicitly
engine.audio.set_master_volume(0.8)            -- clamps to [0.0, 1.0]
engine.audio.set_music_volume(0.5)
engine.audio.set_sfx_volume(1.0)
```

`play_music` stops any currently playing music before starting the new track. All volume setters clamp to `[0.0, 1.0]`. `play_sfx` returns nothing — SFX are fire-and-forget.

### Command queue bridge

Lua closures cannot hold mutable references. Audio commands from scripts are pushed onto a `thread_local! { RefCell<Vec<AudioCommand>> }` queue — the same pattern as `MENU_EVENTS` and `drain_menu_events()`. `AudioManager::poll()` drains it each frame. No channel or cross-thread synchronisation is needed: both the Lua push and the `poll()` drain happen on the main thread.

```rust
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
```

---

## 6. Game Loop Integration

### Engine struct

```rust
pub struct Engine {
    // ... existing fields ...
    pub audio_manager: AudioManager,
}
```

### Poll site

`audio_manager.poll(dt)` is called at the top of `Engine::frame()`, before game updates. Audio is time-sensitive; a frame-early fade update is acceptable, a frame-late one is not.

### Backend selection

`bishop/src/audio/mod.rs` exports a `DefaultAudioBackend` type alias that resolves per feature flag:

```rust
#[cfg(feature = "audio-cpal")]
pub type DefaultAudioBackend = CpalBackend;
```

`EngineBuilder` calls `AudioManager::new::<bishop::audio::DefaultAudioBackend>()`. Adding a console backend is a one-line change: swap the type alias under a new feature flag. Nothing outside `bishop/src/audio/` needs to change.

### cpal buffer conversion

cpal delivers samples as a flat interleaved `&mut [f32]` buffer. `CpalBackend` reinterprets it as `&mut [[f32; 2]]` for oddio using `bytemuck::cast_slice_mut` before invoking `render_fn`. This conversion lives entirely inside `CpalBackend` — `AudioBackend` callers and `AudioManager` never see flat samples.

### Future services

When a second `BackgroundService` arrives (e.g. a network layer), add it as a named field on `Engine` alongside `audio_manager` and call `poll(dt)` on it in the same site. A `Vec<Box<dyn BackgroundService>>` is not needed until the number of services makes individual fields unwieldy.

---

## 7. Module layout

```
bishop/src/audio/
    mod.rs              ← AudioBackend trait + DefaultAudioBackend type alias (public)
    cpal_backend.rs     ← feature: audio-cpal (desktop default); handles [[f32;2]] conversion

engine_core/src/task/
    mod.rs              ← BackgroundTask<T>, BackgroundService trait

engine_core/src/audio/
    mod.rs              ← AudioManager, FadeOut
    command_queue.rs    ← thread_local AudioCommand queue (mirrors MENU_EVENTS pattern)
    loader.rs           ← file resolution, WAV decode, cache population

game/src/scripting/modules/
    audio_module.rs     ← engine.audio Lua registration (matches all other Lua modules)

engine_core/src/constants.rs
    + AUDIO_FOLDER = "audio"

engine_core/src/storage/path_utils.rs
    + audio_folder() -> PathBuf
```

`AudioLuaModule` lives in `game/src/scripting/modules/` alongside `InputModule`, `MenuModule`, and every other Lua module — not in `engine_core`. It pushes `AudioCommand`s into the `thread_local` queue defined in `engine_core/src/audio/command_queue.rs` via a public `push_audio_command(cmd: AudioCommand)` accessor function.

---

## 8. Quick-start test (player.lua)

Drop `games/Demo/Resources/audio/sfx/jump.wav` into the project, then add one line to `player.lua`:

```lua
if engine.input.is_down(input.Space) and is_grounded then
    new_vel.y = -self.public.jump_speed
    engine.audio.play_sfx("sfx/jump")
end
```

No config, no registration. The file resolves automatically.
