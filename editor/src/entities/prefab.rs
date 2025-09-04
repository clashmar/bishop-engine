use core::{
    assets::{asset_manager::AssetManager, sprite::Sprite}, 
    ecs::{component::*, entity::Entity, world_ecs::WorldEcs}
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use macroquad::prelude::*;

/// Human readable name shown in the UI.
pub type PrefabName = String;

/// A serializable description of an entity that can be instantiated.
#[derive(Serialize, Deserialize, Clone)]
pub struct EntityPrefab {
    pub id: Uuid,                   
    pub name: PrefabName,
    pub sprite_path: String,        
    pub components: Vec<ComponentSpec>,
}

impl EntityPrefab {
    pub fn instantiate_entity(
    &self,
    pos: Vec2,
    assets: &mut AssetManager,
    ecs: &mut WorldEcs,
) -> Entity {
        // Load (or reuse) the sprite.
        let sprite_id = futures::executor::block_on(assets.load(&self.sprite_path));
        println!("loaded sprite_id");
        // Base builder – every prefab gets a Position and a Sprite.
        let mut builder = ecs
            .create_entity()
            .with(Position { position: pos })
            .with(Sprite { 
                sprite_id,
                path: self.sprite_path.clone(),
            });

        // Attach the optional components stored in the prefab.
        for spec in &self.components {
            match spec {
                ComponentSpec::Walkable(v) => builder = builder.with(Walkable(*v)),
                ComponentSpec::Solid(v) => builder = builder.with(Solid(*v)),
                ComponentSpec::Damage(d) => builder = builder.with(Damage { amount: *d }),
            }
        }

        builder.finish()
    }

    /// Build an `EntityPrefab` from the current state of an entity.
    pub fn prefab_from_entity(
        ecs: &WorldEcs,
        entity: Entity,
        name: String,
        sprite_path: String,
    ) -> EntityPrefab {
        // Gather optional components.
        let mut comps = Vec::new();
        if let Some(w) = ecs.walkables.get(entity) {
            comps.push(ComponentSpec::Walkable(w.0));
        }
        if let Some(s) = ecs.solids.get(entity) {
            comps.push(ComponentSpec::Solid(s.0));
        }
        if let Some(d) = ecs.damages.get(entity) {
            comps.push(ComponentSpec::Damage(d.amount));
        }

        EntityPrefab {
            id: Uuid::new_v4(),
            name,
            sprite_path,
            components: comps,
        }
    }
}