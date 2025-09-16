// editor/src/playtest/room_playtest.rs
use std::{env, fs, io::Write, path::PathBuf};
use engine_core::world::room::{Room, RoomMetadata};
use engine_core::world::world::World;
use ron::ser::to_string_pretty;
use ron::ser::PrettyConfig;

/// Serialise everything the play‑test binary needs and return the
/// path to the temporary file.
pub fn write_playtest_payload(
    room: &Room,
    room_metadata: &RoomMetadata,
    world: &World,
) -> PathBuf {

    #[derive(serde::Serialize)]
    struct Payload<'a> {
        room: &'a Room,
        room_metadata: &'a RoomMetadata,
        world: &'a World,
    }

    let payload = Payload { room, room_metadata, world };
    let ron = to_string_pretty(&payload, PrettyConfig::default())
        .expect("failed to serialise play‑test payload");

    // Use the OS temporary directory. It will be cleaned up automatically
    let mut tmp = env::temp_dir();
    tmp.push(format!("playtest_{}.ron", uuid::Uuid::new_v4()));
    let mut file = fs::File::create(&tmp).expect("could not create temp play‑test file");
    file.write_all(ron.as_bytes())
        .expect("could not write play‑test payload");
    tmp
}