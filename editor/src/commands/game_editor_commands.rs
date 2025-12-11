// editor/src/commands/game_editor_commands.rs
use engine_core::ui::toast::Toast;
use crate::storage::editor_storage::*;
use engine_core::assets::sprite::SpriteId;
use engine_core::world::world::World;
use crate::with_editor;
use crate::editor::EditorMode;
use engine_core::game::game::Game;
use engine_core::world::world::WorldId;
use macroquad::prelude::*;
use crate::commands::editor_command_manager::EditorCommand;

/// Undo-able move‑entity command.
#[derive(Debug)]
pub struct MoveWorldCmd {
    world_id: WorldId,
    from: Vec2,
    to: Vec2,
}

impl MoveWorldCmd {
    pub fn new(world_id: WorldId, from: Vec2, to: Vec2) -> Self {
        Self {
            world_id,
            from,
            to,
        }
    }

    /// Helper that sets the position of the world.
    fn set_position(game: &mut Game, world_id: WorldId, position: Vec2) {
        if let Some(world) = game.worlds
            .iter_mut()
            .find(|w| w.id == world_id) {
                world.meta.position = position
            }
    }
}

impl EditorCommand for MoveWorldCmd {
    fn execute(&mut self) {
        // Called the first time
        with_editor(|editor| {
            let game = &mut editor.game;
            Self::set_position(game, self.world_id, self.to);
        });
    }

    fn undo(&mut self) {
        // Restore the old position
        with_editor(|editor| {
            let game = &mut editor.game;
            Self::set_position(game, self.world_id, self.from);
        });
    }

    fn mode(&self) -> EditorMode { 
        EditorMode::Game
    }
}

#[derive(Debug)]
pub struct DeleteWorldCmd {
    world_id: WorldId,
    deleted_world: Option<World>,
    prev_current_world: WorldId,
}

impl DeleteWorldCmd {
    pub fn new(game: &mut Game, world_id: WorldId) -> Self {
        Self {
            world_id,
            deleted_world: None,
            prev_current_world: game.current_world_id,
        }
    }
}

impl EditorCommand for DeleteWorldCmd {
    fn execute(&mut self) {
        with_editor(|editor| {
            let game = &mut editor.game;

            // Capture world before deleting
            if let Some(pos) = game.worlds.iter().position(|w| w.id == self.world_id) {
                self.deleted_world = Some(game.worlds.swap_remove(pos));
            }

            // Clean up
            game.delete_world(self.world_id);
            editor.save();
        });
    }

    fn undo(&mut self) {
        with_editor(|editor| {
            // Push the world back into the game
            if let Some(world) = self.deleted_world.take() {
                editor.game.worlds.push(world);
            }

            // Restore previous active world
            editor.game.current_world_id = self.prev_current_world;
            editor.save();
        });
    }

    fn mode(&self) -> EditorMode {
        EditorMode::Game
    }
}

#[derive(Debug)]
pub struct CreateWorldCmd {
    world_id: Option<WorldId>,
}

impl CreateWorldCmd {
    pub fn new() -> Self {
        Self {
            world_id: None,
        }
    }
}

impl EditorCommand for CreateWorldCmd {
    fn execute(&mut self) {
        with_editor(|editor| {
            let game = &mut editor.game;
            let world = create_new_world();
            self.world_id = Some(world.id);
            game.add_world(world);
        });
    }

    fn undo(&mut self) {
        with_editor(|editor| {
            let game = &mut editor.game;
            if let Some(id) = self.world_id.take() {
                game.delete_world(id);
                editor.save();
            }
        });
    }

    fn mode(&self) -> EditorMode {
        EditorMode::Game
    }
}

#[derive(Debug)]
pub struct EditWorldCmd {
    world_id: WorldId,
    /// Values before the edit
    old_name: String,
    old_sprite: Option<SpriteId>,
    /// Values after the edit
    new_name: Option<String>,
    new_sprite: Option<Option<SpriteId>>,
}

impl EditWorldCmd {
    pub fn new(
        world_id: WorldId,
        new_name: Option<String>,
        new_sprite: Option<Option<SpriteId>>,
    ) -> Self {
        Self {
            world_id,
            old_name: String::new(),
            old_sprite: None,
            new_name,
            new_sprite,
        }
    }

    /// Helper that writes the supplied values into the world.
    fn apply(game: &mut Game, world_id: WorldId, name: Option<&str>, sprite: Option<Option<SpriteId>>) {
        if let Some(world) = game.worlds.iter_mut().find(|w| w.id == world_id) {
            if let Some(name) = name {
                world.name = name.to_owned();
            }
            if let Some(sprite_opt) = sprite {
                world.meta.sprite_id = sprite_opt;
            }
        }
    }

    /// Capture the current state of the world.
    fn capture_original_state(&mut self, game: &Game) {
        if let Some(world) = game.worlds.iter().find(|w| w.id == self.world_id) {
            self.old_name = world.name.clone();
            self.old_sprite = world.meta.sprite_id;
        }
    }
}


impl EditorCommand for EditWorldCmd {
    fn execute(&mut self) {
        with_editor(|editor| {
            self.capture_original_state(&editor.game);
        });

        // Apply the new values
        with_editor(|editor| {
            let game = &mut editor.game;
            Self::apply(
                game,
                self.world_id,
                self.new_name.as_deref(),
                self.new_sprite.clone(),
            );

            // Persist the change
            editor.save();
        });
    }

    fn undo(&mut self) {
        // Restore the old values
        with_editor(|editor| {
            let game = &mut editor.game;
            Self::apply(
                game,
                self.world_id,
                Some(&self.old_name),
                Some(self.old_sprite),
            );
            editor.save();
        });
    }

    fn mode(&self) -> EditorMode {
        EditorMode::Game
    }
}

#[derive(Debug)]
pub struct RenameGameCmd {
    pub new_name: String,
    pub old_name: String,
}

impl RenameGameCmd {
    pub fn new(new_name: String, old_name: String) -> Self {
        Self {
            new_name,
            old_name,
        }
    }
}

impl EditorCommand for RenameGameCmd {
    fn execute(&mut self) {
        with_editor(|editor| {
            match rename_game(&mut editor.game, &self.new_name) {
                Ok(()) => {
                    editor.save();
                }
                Err(err) => {
                    editor.toast = Some(Toast::new(
                        &format!("Failed to rename game: {err}"),
                        3.0,
                    ));
                }
            }
        });
    }

    fn undo(&mut self) {
        with_editor(|editor| {
            match rename_game(&mut editor.game, &self.old_name) {
                Ok(()) => {
                    editor.save();
                }
                Err(err) => {
                    editor.toast = Some(Toast::new(
                        &format!("Failed to rename game: {err}"),
                        3.0,
                    ));
                }
            }
        });
    }

    fn mode(&self) -> EditorMode {
        EditorMode::Game
    }
}