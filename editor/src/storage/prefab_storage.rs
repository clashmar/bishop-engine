use crate::entities::prefab::EntityPrefab;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use core::constants::*;
use std::fs;

pub fn prefab_dir(world_id: &Uuid) -> PathBuf {
    Path::new(PREFAB_SAVE_FOLDER)
        .join(world_id.to_string())
        .join("prefabs")
}

pub fn save(prefab: &EntityPrefab, world_id: &Uuid) -> std::io::Result<()> {
    let dir = prefab_dir(world_id);
    fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{}.ron", prefab.name));
    let ron = ron::ser::to_string_pretty(prefab, ron::ser::PrettyConfig::default())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    fs::write(path, ron)
}

pub fn load_all(world_id: &Uuid) -> std::io::Result<Vec<EntityPrefab>> {
    let dir = prefab_dir(world_id);
    if !dir.exists() { return Ok(vec![]); }

    let mut out = Vec::new();
    for entry in fs::read_dir(dir)? {
        let p = entry?.path();
        if p.extension().and_then(|s| s.to_str()) == Some("ron") {
            let data = fs::read_to_string(&p)?;
            let prefab: EntityPrefab = ron::de::from_str(&data)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            out.push(prefab);
        }
    }
    Ok(out)
}