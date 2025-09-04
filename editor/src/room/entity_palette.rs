use core::{
    assets::{
        asset_manager::AssetManager,
        sprite::{Sprite, SpriteId},
    },
    ecs::{
        component::*,                 
        entity::Entity,
        world_ecs::{WorldEcs},
    }, world::room::RoomMetadata,
};
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, FromInto};
use uuid::Uuid;
use crate::{
    entities::prefab::EntityPrefab,
    gui::*,
    storage::prefab_storage,
};

/// One entry that appears in the palette.
#[derive(Serialize, Deserialize)]
struct PrefabEntry {
    id: Uuid,
    sprite_id: SpriteId,
    sprite_path: String,
}

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct EntityPalette {
    #[serde_as(as = "FromInto<[f32; 2]>")]
    pub position: Vec2,
    pub tile_size: f32,
    pub columns: usize,
    pub rows: usize,
    pub selected: usize,               
    entries: Vec<PrefabEntry>,
    #[serde(skip)]
    pub ui: EntityPaletteUi,
    #[serde(skip)]
    pub sprite_ids: Vec<SpriteId>,
    #[serde(skip)]
    create_entity_requested: bool,
    #[serde(skip)]
    edit_requested: bool,
    #[serde(skip)]
    delete_requested: Option<usize>,
}

#[derive(Clone, Default, PartialEq)]
pub enum PrefabPaletteUiMode {
    #[default]
    Create,
    EditEntity,
    EditPrefab,
    SavePrefab, 
}

#[derive(Clone, Default)]
pub struct EntityPaletteUi {
    pub open: bool,
    pub mode: PrefabPaletteUiMode,
    pub edit_initialized: bool,
    pub edit_index: usize,
    pub name: String,
    pub sprite_path: String,
    pub walkable: bool,
    pub solid: bool,
    pub damage: f32,
    pub selected_entity: Option<Entity>,
}

impl EntityPalette {
    pub fn new(
        position: Vec2, 
        tile_size: f32, 
        columns: usize, 
        rows: usize
    ) -> Self {
        Self {
            position,
            tile_size,
            columns,
            rows,
            selected: 0,
            entries: Vec::new(),
            ui: EntityPaletteUi::default(),
            sprite_ids: Vec::new(),
            create_entity_requested: false,
            edit_requested: false,
            delete_requested: None,
        }
    }

    /// Load all prefabs that belong to `world_id`.  Called once when a world
    /// is opened.
    pub fn load_prefabs_from_disk(
        &mut self, 
        world_id: &Uuid,
        asset_manager: &mut AssetManager, 
    ) {
        match prefab_storage::load_all(world_id) {
            Ok(prefs) => {
                self.entries.clear();
                self.sprite_ids.clear();
                for pref in prefs {
                    // Use the *provided* manager, not a fresh one.
                    let sprite_id = futures::executor::block_on(
                        asset_manager.load(&pref.sprite_path),
                    );
                    self.entries.push(PrefabEntry {
                        id: pref.id,
                        sprite_id,
                        sprite_path: pref.sprite_path.clone(),
                    });
                    self.sprite_ids.push(sprite_id);
                }
                self.rows = (self.entries.len() + self.columns - 1) / self.columns;
            }
            Err(e) => eprintln!("Failed to load prefabs: {e}"),
        }
    }

    /// Returns `true` if the mouse click was consumed by the palette UI.
    pub fn handle_click(&mut self, mouse_pos: Vec2) -> bool {
        // Select prefab
        if self.is_mouse_over_grid(mouse_pos) {
            let local = mouse_pos - self.position;
            let col = (local.x / self.tile_size) as usize;
            let row = (local.y / self.tile_size) as usize;
            let idx = row * self.columns + col;
            if idx < self.entries.len() {
                self.selected = idx;
                // Switch UI to prefab‑edit mode.
                self.ui.mode = PrefabPaletteUiMode::EditPrefab;
                self.ui.edit_index = idx;
                self.ui.edit_initialized = true;
                return true;
            }
        }
        false
    }

    #[inline]
    fn is_mouse_over_grid(&self, mouse_pos: Vec2) -> bool {
        let w = self.columns as f32 * self.tile_size;
        let h = self.rows as f32 * self.tile_size;
        Rect::new(self.position.x, self.position.y, w, h).contains(mouse_pos)
    }

    /// Called by `RoomEditor` when the user selects an entity in the scene.
    pub fn enter_entity_edit_mode(&mut self, entity: Entity) {
        self.ui.mode = PrefabPaletteUiMode::EditEntity;
        self.ui.selected_entity = Some(entity);
        self.ui.edit_initialized = true; // fill fields on first draw
        self.ui.open = true;  
    }

    pub fn draw(
        &mut self, 
        asset_manager: &mut AssetManager,
        world_ecs: &mut WorldEcs,
    ) {
        // Draw the grid of prefab thumbnails.
        for (i, _entry) in self.entries.iter().enumerate() {
            let col = i % self.columns;
            let row = i / self.columns;
            let x = self.position.x + col as f32 * self.tile_size;
            let y = self.position.y + row as f32 * self.tile_size;

            let tex = asset_manager.get_texture_from_id(self.sprite_ids[i]);
            draw_texture_ex(
                tex,
                x,
                y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(self.tile_size, self.tile_size)),
                    ..Default::default()
                },
            );

            // Highlight the currently selected prefab.
            if i == self.selected {
                draw_rectangle_lines(x, y, self.tile_size, self.tile_size, 3.0, RED);
            }
        }

        self.draw_add_and_edit_buttons();

        // Modal dialog
        futures::executor::block_on(self.draw_dialog(asset_manager, world_ecs));
    }

    fn draw_add_and_edit_buttons(&mut self) {
        let base_y = self.position.y + (self.rows as f32 * self.tile_size) + 10.0;

        // Add Entity
        let btn_add = Rect::new(
            self.position.x,
            base_y,
            self.tile_size * 2.0,
            30.0,
        );
        if gui_button(btn_add, "Add") {
            self.ui = EntityPaletteUi::default(); // reset fields
            self.ui.open = true;
            self.ui.mode = PrefabPaletteUiMode::Create;
        }

        // Edit Prefab button (only when a prefab exists)
        if !self.entries.is_empty() {
            let btn_edit_prefab = Rect::new(
                btn_add.x + btn_add.w + 5.0,
                base_y,
                btn_add.w,
                btn_add.h,
            );
            if gui_button(btn_edit_prefab, "Edit prefab") {
                self.ui.mode = PrefabPaletteUiMode::EditPrefab;
                self.ui.edit_index = self.selected;
                self.ui.edit_initialized = true;
                self.ui.open = true;
            }
        }
    }

    async fn draw_dialog(
        &mut self, 
        asset_manager: &mut AssetManager,
        world_ecs: &mut WorldEcs,
    ) {
        if !self.ui.open {
            return;
        }

        // Fill fields on edit
        if self.ui.edit_initialized && self.ui.name.is_empty() {
            match self.ui.mode {
                PrefabPaletteUiMode::EditPrefab => {
                    // Load the prefab from disk (the file already exists).
                    let entry = &self.entries[self.ui.edit_index];
                    if let Ok(prefs) = prefab_storage::load_all(&Uuid::nil()) {
                        if let Some(pref) = prefs.into_iter().find(|p| p.id == entry.id) {
                            self.ui.name = pref.name.clone();
                            self.ui.sprite_path = pref.sprite_path.clone();
                            for spec in pref.components {
                                match spec {
                                    ComponentSpec::Walkable(v) => self.ui.walkable = v,
                                    ComponentSpec::Solid(v) => self.ui.solid = v,
                                    ComponentSpec::Damage(v) => self.ui.damage = v,
                                }
                            }
                        }
                    }
                }
                PrefabPaletteUiMode::EditEntity => {
                    // Populate fields from the *live* entity
                    if let Some(entity) = self.ui.selected_entity {
                        // Use a placeholder that the user can edit
                        self.ui.name = "<unnamed>".to_string();

                        // Sprite path – read it from the Sprite component
                        if let Some(_sprite) = world_ecs.sprites.get(entity) {
                            let entry_opt = self
                                .entries
                                .iter()
                                .find(|e| e.id == entity.0); // <-- fallback, rarely used
                            if let Some(entry) = entry_opt {
                                self.ui.sprite_path = entry.sprite_path.clone();
                            }
                        }

                        if let Some(w) = world_ecs.walkables.get(entity) {
                            self.ui.walkable = w.0;
                        }
                        if let Some(s) = world_ecs.solids.get(entity) {
                            self.ui.solid = s.0;
                        }
                        if let Some(d) = world_ecs.damages.get(entity) {
                            self.ui.damage = d.amount;
                        }
                    }
                }
                _ => {}
            }
            self.ui.edit_initialized = false;
        }

        // Dialog layout
        let panel = Rect::new(100.0, 80.0, 340.0, 340.0);
        draw_rectangle(panel.x, panel.y, panel.w, panel.h, Color::new(0., 0., 0., 0.6));
        draw_rectangle_lines(panel.x, panel.y, panel.w, panel.h, 2.0, WHITE);

        // Name field
        let name_rect = Rect::new(panel.x + 10.0, panel.y + 20.0, panel.w - 20.0, 30.0);
        self.ui.name = gui_input_text(name_rect, &self.ui.name);

        // Sprite picker
        let sprite_btn = Rect::new(panel.x + 10.0, panel.y + 60.0, panel.w - 20.0, 30.0);
        if gui_button(sprite_btn, "Pick sprite") {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("PNG images", &["png"])
                .pick_file()
            {
                self.ui.sprite_path = path.to_string_lossy().into_owned();
            }
        }

        // Sprite preview
        if !self.ui.sprite_path.is_empty() {
            let preview_id = asset_manager.load(&self.ui.sprite_path).await;
            let tex = asset_manager.get_texture_from_id(preview_id);
            draw_texture_ex(
                tex,
                panel.x + panel.w - 50.0,
                panel.y + 60.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(40.0, 40.0)),
                    ..Default::default()
                },
            );
        }

        // Component check‑boxes
        let mut walk = self.ui.walkable;
        let mut solid = self.ui.solid;
        let mut dmg = self.ui.damage;

        let cb_walk = Rect::new(panel.x + 10.0, panel.y + 110.0, 20.0, 20.0);
        if gui_checkbox(cb_walk, &mut walk) {
            self.ui.walkable = walk;
        }
        draw_text("Walkable", cb_walk.x + 30.0, cb_walk.y + 15.0, 18.0, WHITE);

        let cb_solid = Rect::new(panel.x + 10.0, panel.y + 140.0, 20.0, 20.0);
        if gui_checkbox(cb_solid, &mut solid) {
            self.ui.solid = solid;
        }
        draw_text("Solid", cb_solid.x + 30.0, cb_solid.y + 15.0, 18.0, WHITE);

        let dmg_rect = Rect::new(panel.x + 10.0, panel.y + 170.0, panel.w - 20.0, 30.0);
        dmg = gui_input_number(dmg_rect, dmg);
        self.ui.damage = dmg.max(0.0);

        // OK / Cancel
        let ok_label = match self.ui.mode {
            PrefabPaletteUiMode::Create => "Create",
            PrefabPaletteUiMode::EditPrefab => "Update",
            PrefabPaletteUiMode::EditEntity => "Apply",
            PrefabPaletteUiMode::SavePrefab => "Save",
        };
        let btn_ok = Rect::new(panel.x + 30.0, panel.y + 220.0, 100.0, 30.0);
        if gui_button(btn_ok, ok_label) {
            match self.ui.mode {
                PrefabPaletteUiMode::Create => {
                    self.create_entity_requested = true; // flag for optional save
                }
                PrefabPaletteUiMode::EditPrefab   => self.edit_requested   = true,
                PrefabPaletteUiMode::EditEntity => {
                    self.apply_entity_edits(world_ecs, asset_manager);
                }
                PrefabPaletteUiMode::SavePrefab => {
                    // Force a save even if the user never pressed “Create” before
                    self.create_entity_requested = true;
                }
            }
            self.ui.open = false;
        }

        let btn_cancel = Rect::new(panel.x + 170.0, panel.y + 220.0, 100.0, 30.0);
        if gui_button(btn_cancel, "Cancel") {
            self.ui.open = false;
        }

        // Delete
        if matches!(self.ui.mode, PrefabPaletteUiMode::EditPrefab) {
            let btn_del = Rect::new(panel.x + 30.0, panel.y + 260.0, 240.0, 30.0);
            if gui_button(btn_del, "Delete") {
                self.delete_requested = Some(self.ui.edit_index);
                self.ui.open = false;
            }
        }
    }

    /// Called once per frame from `RoomEditor::update`.
    pub async fn process_requests(
        &mut self,
        room_metadata: &RoomMetadata,
        world_id: &Uuid,
        asset_manager: &mut AssetManager,
        world_ecs: &mut WorldEcs,
    ) {
        if self.create_entity_requested {
            self.process_create_entity_request(room_metadata, asset_manager, world_ecs).await;
        }
        if self.edit_requested {
            self.process_edit_request(world_id, asset_manager).await;
        }
        if self.delete_requested.is_some() {
            self.process_delete_request(world_id).await;
        }
    }

    async fn process_create_entity_request(
        &mut self,
        room_metadata: &RoomMetadata,
        asset_manager: &mut AssetManager,
        world_ecs: &mut WorldEcs,
    ) {
        self.create_entity_requested = false;

        let pos = room_metadata.position;

        let sprite_id = if !self.ui.sprite_path.is_empty() {
            Some(asset_manager.load(&self.ui.sprite_path).await)
        } else {
            None
        };
        
        // Build entity
        let mut builder = world_ecs
            .create_entity()
            .with(Position { position: pos });

        // Attach the Sprite component if there is a texture
        if let Some(id) = sprite_id {
            builder = builder.with(Sprite { 
                sprite_id: id, 
                path: self.ui.sprite_path.clone(),
            });
        }

        // Optional components
        if self.ui.walkable {
            builder = builder.with(Walkable(true));
        }
        if self.ui.solid {
            builder = builder.with(Solid(true));
        }
        if self.ui.damage > 0.0 {
            builder = builder.with(Damage { amount: self.ui.damage });
        }

        let _entity = builder.finish();
    }

    async fn process_save_prefab_request(
        &mut self,
        world_id: &Uuid,
        assets: &mut AssetManager,
    ) {
        // Build the prefab from the UI fields (same as before)
        let prefab = EntityPrefab {
            id: Uuid::new_v4(),
            name: self.ui.name.clone(),
            sprite_path: self.ui.sprite_path.clone(),
            components: vec![
                ComponentSpec::Walkable(self.ui.walkable),
                ComponentSpec::Solid(self.ui.solid),
                ComponentSpec::Damage(self.ui.damage),
            ],
        };

        // Persist to disk
        if let Err(e) = prefab_storage::save(&prefab, world_id) {
            eprintln!("Failed to save prefab: {e}");
            return;
        }

        // Add a palette entry so the user can reuse it later
        let sprite_id = assets.load(&prefab.sprite_path).await;
        self.entries.push(PrefabEntry {
            id: prefab.id,
            sprite_id,
            sprite_path: prefab.sprite_path.clone(),
        });

        self.sprite_ids.push(sprite_id);
        self.selected = self.entries.len() - 1;
        self.rows = (self.entries.len() + self.columns - 1) / self.columns;
    }

    async fn process_edit_request(
        &mut self,
        world_id: &Uuid,
        asset_manager: &mut AssetManager,
    ) {
        self.edit_requested = false;

        let idx = self.ui.edit_index;
        let entry = &mut self.entries[idx];

        // Reload sprite (in case the path changed).
        let sprite_id = asset_manager.load(&self.ui.sprite_path).await;

        // Build the updated prefab.
        let prefab = EntityPrefab {
            id: entry.id, // keep the same stable ID
            name: self.ui.name.clone(),
            sprite_path: self.ui.sprite_path.clone(),
            components: vec![
                ComponentSpec::Walkable(self.ui.walkable),
                ComponentSpec::Solid(self.ui.solid),
                ComponentSpec::Damage(self.ui.damage),
            ],
        };

        // Overwrite the file (same filename because we use the prefab name).
        if let Err(e) = prefab_storage::save(&prefab, world_id) {
            eprintln!("Failed to update prefab: {e}");
            return;
        }

        // Update UI entry.
        entry.sprite_id = sprite_id;
        entry.sprite_path = prefab.sprite_path.clone();
        self.sprite_ids[idx] = sprite_id;
    }

    async fn process_delete_request(&mut self, world_id: &Uuid) {
        if let Some(idx) = self.delete_requested.take() {
            let entry = self.entries.remove(idx);
            self.sprite_ids.remove(idx);

            // Delete the file on disk.
            let file = prefab_storage::prefab_dir(world_id)
                .join(format!("{}.ron", entry.id));
            let _ = std::fs::remove_file(file); // ignore error

            // Keep selection in range.
            self.selected = self.entries.len().saturating_sub(1);
            self.rows = (self.entries.len() + self.columns - 1) / self.columns;
        }
    }

    /* ---------------------------------------------------------------------- */
    /*                         Helper – entity edit                           */
    /* ---------------------------------------------------------------------- */

    /// Called from the dialog when the user presses **Apply** in EntityEdit mode.
    pub fn apply_entity_edits(&mut self, ecs: &mut WorldEcs, assets: &mut AssetManager) {
        if let Some(ent) = self.ui.selected_entity {
            // ---- Walkable -------------------------------------------------
            if self.ui.walkable {
                ecs.walkables.insert(ent, Walkable(true));
            } else {
                ecs.walkables.remove(ent);
            }

            // ---- Solid ----------------------------------------------------
            if self.ui.solid {
                ecs.solids.insert(ent, Solid(true));
            } else {
                ecs.solids.remove(ent);
            }

            // ---- Damage ---------------------------------------------------
            if self.ui.damage > 0.0 {
                ecs.damages.insert(ent, Damage { amount: self.ui.damage });
            } else {
                ecs.damages.remove(ent);
            }

            // ---- Sprite (optional) ----------------------------------------
            if !self.ui.sprite_path.is_empty() {
                // Load (or reuse) the texture.
                let new_sprite_id = futures::executor::block_on(assets.load(&self.ui.sprite_path));
                ecs.sprites.insert(ent, Sprite { 
                    sprite_id: new_sprite_id,
                    path: self.ui.sprite_path.clone(),
                });
            }
        }
    }

    /// Retrieve the runtime prefab that matches the currently
    /// selected grid entry.
    pub fn selected_prefab(&self, world_id: &Uuid) -> Option<EntityPrefab> {
        if self.entries.is_empty() {
            return None;
        }
        let entry = &self.entries[self.selected];
        prefab_storage::load_all(world_id)
            .ok()?
            .into_iter()
            .find(|p| p.id == entry.id)
    }
}