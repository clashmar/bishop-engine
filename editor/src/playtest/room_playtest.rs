// editor/src/playtest/room_playtest.rs
use engine_core::storage::path_utils::game_binary_dir;
use std::io::{Error, ErrorKind};
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
) -> PathBuf {

    #[derive(serde::Serialize)]
    struct Payload<'a> {
        room: &'a Room,
        game: &'a Game,
    }

    let payload = Payload { room, game };
    let ron = to_string_pretty(&payload, PrettyConfig::default())
        .expect("failed to serialise playtest payload");

    // Use the OS temporary directory. It will be cleaned up automatically
    let mut tmp = env::temp_dir();
    tmp.push(format!("playtest_{}.ron", uuid::Uuid::new_v4()));
    let mut file = fs::File::create(&tmp).expect("could not create temp playtest file");
    file.write_all(ron.as_bytes())
        .expect("could not write playtest payload");
    tmp
}

/// Return the absolute path to the game executable.
/// If in dev mode, builds the binary first.
pub async fn resolve_playtest_binary() -> std::io::Result<PathBuf> {    
    // Choose the correct binary name for the platform
    #[cfg(target_os = "windows")]
    let exe_name = "game-playtest.exe";
    #[cfg(not(target_os = "windows"))]
    let exe_name = "game-playtest";

    // Release mode
    if !cfg!(debug_assertions) {
        if let Some(mut exe_path) = game_binary_dir() {
            exe_path.push(exe_name);
            return Ok(exe_path);
        } else {
            return Err(Error::new(
                ErrorKind::Other,
                "Could not find game binary directory.",
            ));
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