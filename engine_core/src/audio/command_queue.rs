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
