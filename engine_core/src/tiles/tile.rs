// engine_core/src/tiles/tile_def.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::assets::sprite::SpriteId;

/// Opaque identifier used by the editor and by the TileMap.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TileDefId(pub Uuid);

/// A list of component adding closures.
#[derive(Clone, Serialize, Deserialize)]
pub struct TileDef {
    /// SpriteId for the tile.
    pub sprite_id: SpriteId,
    /// The list of tile components that the tile has.
    pub components: Vec<TileComponent>,
}

/// Serialisable description of a component.
#[derive(PartialEq, Clone, Serialize, Deserialize)]
pub enum TileComponent {
    Walkable(bool),
    Solid(bool),
    Damage(f32),
}