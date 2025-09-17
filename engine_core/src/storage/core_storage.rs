// engine_core/src/storage/core_storage.rs
use uuid::Uuid;
use crate::{
    constants::WORLD_SAVE_FOLDER,
    world::world::World,
};
use std::{
    collections::HashMap,
    fs, io,
    path::Path,
    time::SystemTime,
};

pub type WorldIndex = HashMap<Uuid, String>;

/// Load a whole `World` (including its `WorldEcs`) from disk.
pub fn load_world_by_id(id: &Uuid) -> io::Result<World> {
    let path = Path::new(WORLD_SAVE_FOLDER)
        .join(id.to_string())
        .join("world.ron");
    let ron_string = fs::read_to_string(path)?;
    ron::from_str(&ron_string).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}

/// Return the UUID of the most‑recently‑modified world folder.
pub fn most_recent_world_id() -> Option<Uuid> {
    let root = Path::new(WORLD_SAVE_FOLDER);
    let mut best: Option<(Uuid, SystemTime)> = None;
    for entry in fs::read_dir(root).ok()? {
        let entry = entry.ok()?;
        if !entry.path().is_dir() {
            continue;
        }
        if let Ok(uuid) = Uuid::parse_str(&entry.file_name().to_string_lossy()) {
            if let Ok(mod_time) = entry.metadata().ok()?.modified() {
                match best {
                    None => best = Some((uuid, mod_time)),
                    Some((_, t)) if mod_time > t => best = Some((uuid, mod_time)),
                    _ => {}
                }
            }
        }
    }
    best.map(|(id, _)| id)
}