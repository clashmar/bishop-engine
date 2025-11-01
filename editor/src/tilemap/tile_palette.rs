// editor/src/tilemap/tile_palette.rs
use std::collections::VecDeque;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use engine_core::{global::tile_size, ui::widgets::*};
use serde_with::serde_as;
use engine_core::{
    assets::{asset_manager::AssetManager, sprite::SpriteId},
    ecs::world_ecs::WorldEcs,
    tiles::{
        tile_def::{TileComponentSpec, TileDef, TileDefId}
    },
};

#[derive(Serialize, Deserialize)]
pub struct PaletteEntry {
    def_id: TileDefId,
    sprite_id: SpriteId,
    sprite_path: String,
}

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct TilePalette {
    pub tile_size: f32,
    pub columns: usize,
    pub rows: usize,
    pub selected_index: usize,
    pub entries: Vec<PaletteEntry>,
    #[serde(skip)]
    pub ui: TilePaletteUi,
    #[serde(skip)]
    pub sprite_ids: Vec<SpriteId>,
    #[serde(skip)]
    command_queue: VecDeque<PaletteCmd>,
}

enum PaletteCmd { Create, Edit, Delete(usize) }

#[derive(Clone, Default, PartialEq)]
pub enum TilePaletteUiMode {
    #[default]
    Create,
    Edit,
}

#[derive(Clone, Default)]
pub struct TilePaletteUi {
    pub open: bool,
    pub mode: TilePaletteUiMode,
    pub edit_initialized: bool,
    pub edit_index: usize, 
    pub name: String,
    pub sprite_path: String,
    pub walkable: bool,
    pub solid: bool,
    pub damage: f32,
}

impl TilePalette {
    pub fn new() -> Self {
        Self {
            ui: TilePaletteUi::default(),
            tile_size: tile_size(),
            columns: 1,
            rows: 0,
            selected_index: 0,
            entries: Vec::new(),
            sprite_ids: Vec::new(),
            command_queue: VecDeque::new(),
        }
    }

    pub async fn update(
        &mut self,
        world_ecs: &mut WorldEcs,
        asset_manager: &mut AssetManager,
    ) {
        while let Some(cmd) = self.command_queue.pop_front() {
            match cmd {
                PaletteCmd::Create => self.create_tile(world_ecs, asset_manager).await,
                PaletteCmd::Edit => self.edit_tile(world_ecs, asset_manager).await,
                PaletteCmd::Delete(i) => self.delete_tile(i, world_ecs).await,
            }
        }
    }

    /// Returns the currently selected TileDefId, or `None` when the palette
    /// is still empty.
    #[inline]
    pub fn selected_def_opt(&self) -> Option<TileDefId> {
        self.entries.get(self.selected_index).map(|e| e.def_id)
    }

    /// Returns the currently selected SpriteId, or `None` when the palette
    /// is still empty.
    #[inline]
    pub fn selected_sprite_opt(&self) -> Option<SpriteId> {
        self.sprite_ids.get(self.selected_index).copied()
    }

    /// Returns the path of the currently selected sprite, or `None` when the
    /// palette is empty (or the index is out of range).
    #[inline]
    pub fn selected_path_opt(&self) -> Option<&str> {
        self.entries.get(self.selected_index).map(|e| e.sprite_path.as_str())
    }

    /// Loads every sprite that belongs to the palette and fills the
    /// `sprite_ids` / `sprite_paths` vectors.
    pub async fn rebuild_runtime(&mut self, asset_manager: &mut AssetManager) {
        self.sprite_ids.clear();

        for entry in &self.entries {
            let tex_id = match asset_manager.init_texture(&entry.sprite_path).await {
                Ok(id) => id,
                Err(_) => SpriteId(Uuid::nil()),
            };
            self.sprite_ids.push(tex_id);
        }
    }

    pub fn draw(
        &mut self,
        rect: Rect,
        asset_manager: &mut AssetManager,
        world_ecs: &WorldEcs,
    ) {
        // Draw grid
        for i in 0..self.entries.len() {
            let col = i % self.columns;
            let row = i / self.columns;
            let y = rect.y + (row as f32 * tile_size());

            // Skip rows that are completely outside the visible area
            if y + self.tile_size < rect.y
                || y > rect.y + tile_size() * 5.0
            {
                continue;
            }

            let x = rect.x + col as f32 * self.tile_size;

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
            if i == self.selected_index {
                draw_rectangle_lines(x, y, self.tile_size, self.tile_size, 3.0, RED);
            }
        }

        futures::executor::block_on(self.draw_tile_dialog(asset_manager, world_ecs));
    }

    /// Called from `TileMapEditor::handle_ui_click` when the mouse
    /// is over the palette area. Returns `true` if the click was
    /// consumed (i.e. user selected a tile).
    pub fn handle_click(&mut self, mouse_pos: Vec2, rect: Rect) -> bool {
        if !Rect::new(
            rect.x, 
            rect.y,
            self.columns as f32 * self.tile_size,
            self.rows as f32 * self.tile_size
        )
            .contains(mouse_pos) {
            return false;
        }

        let local_x = mouse_pos.x - rect.x;
        let local_y = mouse_pos.y - rect.y;
        let col = (local_x / self.tile_size) as usize;
        let row = (local_y / self.tile_size) as usize;
        let idx = row * self.columns + col;
        if idx < self.entries.len() {
            self.selected_index = idx;
            return true;
        }
        false
    }

    async fn draw_tile_dialog(&mut self, asset_manager: &mut AssetManager, world_ecs: &WorldEcs) {
        if !self.ui.open {
            return;
        }

        if self.ui.edit_initialized && self.ui.name.is_empty() {
            let entry = &self.entries[self.ui.edit_index];
            
            let def = world_ecs.tile_defs
                .get(&entry.def_id)
                .expect("def must exist");

            self.ui.name = def.name.clone();
            self.ui.sprite_path = entry.sprite_path.clone();
            // Walk through the component specs
            for spec in &def.components {
                match spec {
                    TileComponentSpec::Walkable(v) => self.ui.walkable = *v,
                    TileComponentSpec::Solid(v)    => self.ui.solid    = *v,
                    TileComponentSpec::Damage(d)  => self.ui.damage   = *d,
                }
            }
            self.ui.edit_initialized = false;
        }

        // Background panel
        let panel = Rect::new(100., 80., 300., 260.);
        draw_rectangle(panel.x, panel.y, panel.w, panel.h, Color::new(0., 0., 0., 0.6));
        draw_rectangle_lines(panel.x, panel.y, panel.w, panel.h, 2., WHITE);

        // Sprite selector
        let sprite_rect = Rect::new(panel.x + 10., panel.y + 60., panel.w - 20., 30.);
        if gui_button(sprite_rect, "Pick sprite") {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("PNG images", &["png"])
                .pick_file()
            {
                self.ui.sprite_path = path.to_string_lossy().into_owned();
            }
        }
        
        // Preview
        if !self.ui.sprite_path.is_empty() {
            let preview_id = match asset_manager.init_texture(&self.ui.sprite_path).await {
                Ok(id) => id,
                Err(_) => SpriteId(Uuid::nil()),
            };

            let tex = asset_manager.get_texture_from_id(preview_id);
            draw_texture_ex(
                tex,
                panel.x + panel.w - 50.,
                panel.y + 60.,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(40., 40.)),
                    ..Default::default()
                },
            );
        }

        // Component check‑boxes
        let mut walk = self.ui.walkable;
        let mut solid = self.ui.solid;

        let cb_walk = Rect::new(panel.x + 10., panel.y + 110., 20., 20.);
        if gui_checkbox(cb_walk, &mut walk) {
            self.ui.walkable = walk;
        }
        draw_text("Walkable", cb_walk.x + 30., cb_walk.y + 15., 18., WHITE);

        let cb_solid = Rect::new(panel.x + 10., panel.y + 140., 20., 20.);
        if gui_checkbox(cb_solid, &mut solid) {
            self.ui.solid = solid;
        }
        draw_text("Solid", cb_solid.x + 30., cb_solid.y + 15., 18., WHITE);

        let btn_label = match self.ui.mode {
            TilePaletteUiMode::Create => { "Create" },
            TilePaletteUiMode::Edit => { "Update" }, 
        };

        // Create/Update
        let btn_ok = Rect::new(panel.x + 30., panel.y + 220., 100., 30.);
        if gui_button(btn_ok, btn_label) {
            // Add the request to the queue, it will be excecuted next frame
            let cmd = match self.ui.mode {
                TilePaletteUiMode::Create => PaletteCmd::Create,
                TilePaletteUiMode::Edit   => PaletteCmd::Edit,
            };
            self.command_queue.push_back(cmd);
            self.ui.open = false;
        }

        // Cancel
        let btn_cancel = Rect::new(panel.x + 170., panel.y + 220., 100., 30.);
        if gui_button(btn_cancel, "Cancel") {
            self.ui.open = false;
        }

        // Draw delete button if in edit mode
        if self.ui.mode == TilePaletteUiMode::Edit {
            let btn_del = Rect::new(panel.x + 30., panel.y + 260., 240., 30.);
            if gui_button(btn_del, "Delete") {
                //Add the request to the queue
                let cmd = PaletteCmd::Delete(self.ui.edit_index);
                self.command_queue.push_back(cmd);
                self.ui.open = false;
            }
        }
    }

    pub async fn create_tile(
        &mut self,
        world_ecs: &mut WorldEcs,
        asset_manager: &mut AssetManager,
    ) {
        // Load sprite
        let sprite_id = match asset_manager.init_texture(&self.ui.sprite_path).await {
            Ok(id) => id,
            Err(_) => SpriteId(Uuid::nil()),
        };

        // Build TileDef
        let mut comps = vec![
            TileComponentSpec::Walkable(self.ui.walkable),
            TileComponentSpec::Solid(self.ui.solid),
        ];
        
        if self.ui.damage > 0.0 {
            comps.push(TileComponentSpec::Damage(self.ui.damage));
        }
        
        let def = TileDef {
            name: self.ui.name.clone(),
            components: comps,
        };

        // Insert the definition into the world map.
        let def_id = TileDefId(Uuid::new_v4());
        world_ecs.tile_defs.insert(def_id, def);

        // Persist the palette entry
        self.entries.push(PaletteEntry {
            def_id,
            sprite_id,
            sprite_path: self.ui.sprite_path.clone(),
        });

        self.sprite_ids.push(sprite_id);

        // Auto‑select the newly created tile
        self.selected_index = self.entries.len() - 1;

        // Grow the UI grid
        let needed = self.entries.len();
        self.rows = (needed + self.columns - 1) / self.columns; // ceil‑div
    }

    pub async fn edit_tile(
        &mut self,
        world_ecs: &mut WorldEcs,
        asset_manager: &mut AssetManager,
    ) {
        // Load sprite
        let sprite_id = match asset_manager.init_texture(&self.ui.sprite_path).await {
            Ok(id) => id,
            Err(_) => SpriteId(Uuid::nil()),
        };

        // Build TileDef
        let mut comps = vec![
            TileComponentSpec::Walkable(self.ui.walkable),
            TileComponentSpec::Solid(self.ui.solid),
        ];
        if self.ui.damage > 0.0 {
            comps.push(TileComponentSpec::Damage(self.ui.damage));
        }
        let def = TileDef {
            name: self.ui.name.clone(),
            components: comps,
        };

        // Overwrite the existing definition.
        let entry = &self.entries[self.ui.edit_index];
        world_ecs.tile_defs.insert(entry.def_id, def);

        // Update the palette entry (path + sprite id may have changed).
        self.entries[self.ui.edit_index].sprite_path = self.ui.sprite_path.clone();
        self.entries[self.ui.edit_index].sprite_id = sprite_id;
    }

    pub async fn delete_tile(&mut self, idx: usize, world_ecs: &mut WorldEcs) {
        // Remove the definition from the world
        let def_id = self.entries[idx].def_id;
        world_ecs.tile_defs.remove(&def_id);

        // Remove palette entry and sprite id
        self.entries.remove(idx);
        self.sprite_ids.remove(idx);

        // Adjust selected index safely
        self.selected_index = self.entries.len().saturating_sub(1);
        
        // Re‑compute rows
        self.rows = (self.entries.len() + self.columns - 1) / self.columns;
    }

    /// The current height of the palette.
    pub fn height(&self) -> f32 {
        self.rows as f32 * self.tile_size
    }

    /// Called after self.columns changes.
    fn recompute_rows(&mut self) {
        self.rows = if self.columns == 0 {
            0
        } else {
            (self.entries.len() + self.columns - 1) / self.columns
        };
    }

    /// Set the column count based on an available width.
    pub fn set_columns_for_width(&mut self, available_width: f32) {
        let cols = (available_width / self.tile_size).floor() as usize;
        self.columns = cols.max(1); // at least one column
        self.recompute_rows();
    }
}