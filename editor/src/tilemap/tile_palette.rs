use core::{
    assets::{asset_manager::AssetManager, sprite::SpriteId},
    ecs::world_ecs::WorldEcs,
    tiles::{
        tile_def::{TileComponentSpec, TileDef, TileDefId}
    },
};
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::gui::*;
use serde_with::serde_as;
use serde_with::FromInto;

#[derive(Serialize, Deserialize)]
struct PaletteEntry {
    def_id: TileDefId,
    sprite_id: SpriteId,
    sprite_path: String,
}

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct TilePalette {
    #[serde_as(as = "FromInto<[f32; 2]>")]
    pub position: Vec2,
    pub tile_size: f32,
    pub columns: usize,
    pub rows: usize,
    pub selected_index: usize,
    entries: Vec<PaletteEntry>,
    #[serde(skip)]
    pub ui: TilePaletteUi,
    #[serde(skip)]
    pub sprite_ids: Vec<SpriteId>,
    #[serde(skip)]
    create_requested: bool,
    edit_requested: bool,
    delete_requested: Option<usize>,
}

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
    pub fn new(position: Vec2, tile_size: f32, columns: usize, rows: usize) -> Self {
        Self {
            ui: TilePaletteUi::default(),
            position,
            tile_size,
            columns,
            rows,
            selected_index: 0,
            entries: Vec::new(),
            sprite_ids: Vec::new(),
            create_requested: false,
            edit_requested: false,
            delete_requested: None,
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
            let tex_id = asset_manager.load(&entry.sprite_path).await;
            self.sprite_ids.push(tex_id);
        }
    }

    pub fn draw(
        &mut self,
        asset_manager: &mut AssetManager,
        world_ecs: &WorldEcs,
    ) {
        for i in 0..self.entries.len() {
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

            if i == self.selected_index {
                draw_rectangle_lines(x, y, self.tile_size, self.tile_size, 3.0, RED);
            }
        }

        self.draw_add_and_edit_buttons();

        // This is an async function but we don’t await here.
        // It only sets create_requested; the actual work is done
        // later by `process_create_request`.
        futures::executor::block_on(self.draw_tile_dialog(asset_manager, world_ecs));
    }

    /// Called from `TileMapEditor::handle_ui_click` when the mouse
    /// is over the palette area. Returns `true` if the click was
    /// consumed (i.e. user selected a tile).
    pub fn handle_click(&mut self, mouse_pos: Vec2, _camera: &Camera2D) -> bool {
        if !self.is_mouse_over(mouse_pos, _camera) {
            return false;
        }
        let local_x = mouse_pos.x - self.position.x;
        let local_y = mouse_pos.y - self.position.y;
        let col = (local_x / self.tile_size) as usize;
        let row = (local_y / self.tile_size) as usize;
        let idx = row * self.columns + col;
        if idx < self.entries.len() {
            self.selected_index = idx;
            return true;
        }
        false
    }

    #[inline]
    fn is_mouse_over(&self, mouse_pos: Vec2, _camera: &Camera2D) -> bool {
        let w = self.columns as f32 * self.tile_size;
        let h = self.rows as f32 * self.tile_size;
        Rect::new(self.position.x, self.position.y, w, h).contains(mouse_pos)
    }

    /// Draw the “Add” button. When a tile is selected,
    /// draw the “Edit” button as well.
    fn draw_add_and_edit_buttons(&mut self) {
        // Add Tile
        let btn_add = Rect::new(
            self.position.x,
            self.position.y + (self.rows as f32 * self.tile_size) + 10.,
            self.tile_size * 2.,
            30.,
        );

        if gui_button(btn_add, "Add") {
            self.ui = TilePaletteUi::default(); // reset fields
            self.ui.open = true;
            self.ui.mode = TilePaletteUiMode::Create;
        }

        // Draw Edit button if a tile is selected
        if !self.entries.is_empty() {
            let btn_edit = Rect::new(
                btn_add.x + btn_add.w + 5.0,              
                btn_add.y,
                btn_add.w,
                btn_add.h,
            );
            if gui_button(btn_edit, "Edit") {
                // Initialise the dialog with the currently selected tile.
                self.ui.mode = TilePaletteUiMode::Edit;
                self.ui.edit_index = self.selected_index;
                self.ui.edit_initialized = true; // will fill fields on first draw
                self.ui.open = true;
            }
        }
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

        // Name
        let name_rect = Rect::new(panel.x + 10., panel.y + 20., panel.w - 20., 30.);
        self.ui.name = gui_input_text(name_rect, &self.ui.name);

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
            let preview_id = asset_manager.load(&self.ui.sprite_path).await;
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
        let mut dmg = self.ui.damage;

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

        let dmg_rect = Rect::new(panel.x + 10., panel.y + 170., panel.w - 20., 30.);
        dmg = gui_input_number(dmg_rect, dmg);
        self.ui.damage = dmg.max(0.0);

        let btn_label = match self.ui.mode {
            TilePaletteUiMode::Create => { "Create" },
            TilePaletteUiMode::Edit => { "Update" }, 
        };

        // Create/Update
        let btn_ok = Rect::new(panel.x + 30., panel.y + 220., 100., 30.);
        if gui_button(btn_ok, btn_label) {
            // Signal the request – the editor will pick it up next frame.
            match self.ui.mode {
                TilePaletteUiMode::Create => {
                    self.create_requested = true;
                },
                TilePaletteUiMode::Edit => {
                    self.edit_requested = true;
                }
            }
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
                self.delete_requested = Some(self.ui.edit_index);
                self.ui.open = false;
            }
        }
    }

    pub async fn process_requests(
        &mut self, 
        world_ecs: &mut WorldEcs,
        asset_manager: &mut AssetManager,
    ) {
        if self.create_requested {
            self.process_create_request(world_ecs, asset_manager).await;
        }
        if self.edit_requested {
            self.process_edit_request(world_ecs, asset_manager).await;
        }
        if self.delete_requested.is_some() {
            self.process_delete_request(world_ecs).await;
        }
    }

    pub async fn process_create_request(
        &mut self,
        world_ecs: &mut WorldEcs,
        asset_manager: &mut AssetManager,
    ) {
        if !self.create_requested {
            return;
        }
        // Reset early to avoid double-processing.
        self.create_requested = false;

        // Load sprite
        let sprite_id = asset_manager.load(&self.ui.sprite_path).await;

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

    pub async fn process_edit_request(
        &mut self,
        world_ecs: &mut WorldEcs,
        asset_manager: &mut AssetManager,
    ) {
        if !self.edit_requested {
            return;
        }
        // Reset early to avoid double-processing.
        self.edit_requested = false;

        // Load sprite
        let sprite_id = asset_manager.load(&self.ui.sprite_path).await;

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

    pub async fn process_delete_request(&mut self, world_ecs: &mut WorldEcs) {
        if let Some(idx) = self.delete_requested.take() {
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
    }
}