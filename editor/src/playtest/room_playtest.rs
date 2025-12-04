// editor/src/playtest/room_playtest.rs
use crate::editor_assets::editor_assets::*;
use crate::storage::editor_storage::*;
use std::io::{Error, ErrorKind};
use std::io;
use std::process::Command;
use std::{env, fs, io::Write, path::PathBuf};
use engine_core::game::game::Game;
use engine_core::world::room::Room;
use ron::ser::to_string_pretty;
use ron::ser::PrettyConfig;

/// Serialise everything the play‑test binary needs and return the
/// path to the temporary file.
pub fn write_playtest_payload(
    room: &Room,
    game: &Game,
) -> io::Result<PathBuf> {

    #[derive(serde::Serialize)]
    struct Payload<'a> {
        room: &'a Room,
        game: &'a Game,
    }
    
    let payload = Payload { room, game };

    let ron = to_string_pretty(&payload, PrettyConfig::default())
        .map_err(|e| io::Error::new(ErrorKind::Other, format!("Could not serialize payload: {e}")))?;

    // Use the OS temporary directory. It will be cleaned up automatically
    let mut temp_dir = env::temp_dir();

    temp_dir.push(format!("playtest_{}.ron", uuid::Uuid::new_v4()));

    let mut file = fs::File::create(&temp_dir)?;

    file.write_all(ron.as_bytes())?;

    Ok(temp_dir)
}

/// Return the absolute path to the game executable.
/// If in dev mode, builds the binary first.
pub async fn resolve_playtest_binary() -> io::Result<PathBuf> {    
    // Choose the correct binary name for the platform
    #[cfg(target_os = "windows")] 
    let exe_name = "game-playtest.exe";
    #[cfg(target_os = "macos")]
    let exe_name = "game-playtest";

    // Release mode
    if !cfg!(debug_assertions) {
        #[cfg(target_os = "windows")] {
            // Write PLAYTEST_EXE to a temp file and return path
            return write_to_app_dir(exe_name, PLAYTEST_EXE);
        }
        #[cfg(target_os = "macos")] {
            return write_to_app_dir(exe_name, PLAYTEST_BIN);
        }
    }

    // Dev mode
    let mut exe_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    exe_path.pop();
    exe_path.push("target");
    exe_path.push("debug");
    exe_path.push(exe_name);

    // Run `cargo build -p game-playtest`
    let mut cmd = Command::new("cargo");
    cmd.arg("build")
        .arg("-p")
        .arg("game")
        .arg("--bin")
        .arg("game-playtest");

    // Wait for the build to complete
    let status = cmd.status()?;

    if status.success() {
        Ok(exe_path)
    } else {
        Err(Error::new(
            ErrorKind::Other,
            "Playtest build failed.",
        ))
    }
}