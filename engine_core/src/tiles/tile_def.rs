// engine_core/src/tiles/tile_def.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::ecs::{component::{Damage, Solid, Walkable}, entity::EntityBuilder};

/// Opaque identifier used by the editor and by the TileMap.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TileDefId(pub Uuid);

/// A list of component adding closures.
#[derive(Clone, Serialize, Deserialize)]
pub struct TileDef {
    /// Human‑readable name that appears in the palette UI.
    pub name: String,
    /// The list of component specs that will be added to the tile entity.
    pub components: Vec<TileComponentSpec>,
}

/// Serialisable description of a component.
#[derive(Clone, Serialize, Deserialize)]
pub enum TileComponentSpec {
    Walkable(bool),
    Solid(bool),
    Damage(f32),
}

impl TileDef {
    /// Apply the spec list to an EntityBuilder.
    pub fn apply<'a>(&'a self, mut builder: EntityBuilder<'a>) -> EntityBuilder<'a> {
        for spec in &self.components {
            match *spec {
                TileComponentSpec::Walkable(v) => {
                    builder = builder.with(Walkable(v));
                }
                TileComponentSpec::Solid(v) => {
                    builder = builder.with(Solid(v));
                }
                TileComponentSpec::Damage(d) => {
                    builder = builder.with(Damage { amount: d });
                }
            }
        }
        builder
    }
}