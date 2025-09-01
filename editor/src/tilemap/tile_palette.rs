use core::{
    assets::{asset_manager::AssetManager, sprites::SpriteId},
    ecs::world_ecs::WorldEcs,
    tiles::{
        tile_def::{TileComponentSpec, TileDef, TileDefId}
    },
};
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use crate::gui::*;
use serde_with::serde_as;
use serde_with::FromInto;

#[derive(Serialize, Deserialize)]
struct PaletteEntry {
    def_id: TileDefId,
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
    pub sprite_paths: Vec<String>,
    #[serde(skip)]
    create_requested: bool,
}

#[derive(Clone, Default)]
pub struct TilePaletteUi {
    pub open: bool,
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
            sprite_paths: Vec::new(),
            create_requested: false,
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
        self.sprite_paths.get(self.selected_index).map(|s| s.as_str())
    }

    /// Loads every sprite that belongs to the palette and fills the
    /// `sprite_ids` / `sprite_paths` vectors.
    pub async fn rebuild_runtime(&mut self, assets: &mut AssetManager) {
        self.sprite_ids.clear();
        self.sprite_paths.clear();

        for entry in &self.entries {
            let tex_id = assets.load(&entry.sprite_path).await;
            self.sprite_ids.push(tex_id);
            self.sprite_paths.push(entry.sprite_path.clone());
        }
    }

    pub fn draw(&mut self, _camera: &Camera2D, assets: &mut AssetManager) {
        for i in 0..self.entries.len() {
            let col = i % self.columns;
            let row = i / self.columns;
            let x = self.position.x + col as f32 * self.tile_size;
            let y = self.position.y + row as f32 * self.tile_size;

            let tex = assets.get(self.sprite_ids[i]);
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

        self.draw_add_tile_button();

        // This is an async function but we don’t await here.
        // It only sets create_requested; the actual work is done
        // later by `process_create_request`.
        futures::executor::block_on(self.draw_add_tile_dialog(assets));
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

    fn draw_add_tile_button(&mut self) {
        let btn = Rect::new(
            self.position.x,
            self.position.y + (self.rows as f32 * self.tile_size) + 10.,
            self.tile_size * 2.,
            30.,
        );
        if gui_button(btn, "Add Tile") {
            self.ui = TilePaletteUi::default(); // reset fields
            self.ui.open = true;
        }
    }

    async fn draw_add_tile_dialog(&mut self, assets: &mut AssetManager) {
        if !self.ui.open {
            return;
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
            let tex_id = assets.load(&self.ui.sprite_path).await;
            let tex = assets.get(tex_id);
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

        // Confirm / Cancel
        let btn_ok = Rect::new(panel.x + 30., panel.y + 220., 100., 30.);
        let btn_cancel = Rect::new(panel.x + 170., panel.y + 220., 100., 30.);
        if gui_button(btn_ok, "Create") {
            // Signal the request – the editor will pick it up next frame.
            self.create_requested = true;
            self.ui.open = false;
        }
        if gui_button(btn_cancel, "Cancel") {
            self.ui.open = false;
        }
    }

    pub async fn process_create_request(
        &mut self,
        world_ecs: &mut WorldEcs,
        assets: &mut AssetManager,
    ) {
        if !self.create_requested {
            return;
        }
        // Reset early to avoid double-processing.
        self.create_requested = false;

        // Load sprite
        let sprite_id = assets.load(&self.ui.sprite_path).await;

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

        // Store the definition in the world and remember the IDs
        let def_id = TileDefId(world_ecs.tile_defs.len());
        world_ecs.tile_defs.push(def);

        // Persist the palette entry
        self.entries.push(PaletteEntry {
            def_id,
            sprite_path: self.ui.sprite_path.clone(),
        });
        self.sprite_ids.push(sprite_id);
        self.sprite_paths.push(self.ui.sprite_path.clone());

        // Auto‑select the newly created tile
        self.selected_index = self.entries.len() - 1;

        // Grow the UI grid
        let needed = self.entries.len();
        self.rows = (needed + self.columns - 1) / self.columns; // ceil‑div
    }
}