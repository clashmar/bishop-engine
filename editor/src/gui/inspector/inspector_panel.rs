// editor/src/gui/inspector/inspector_panel.rs
use crate::gui::inspector::room_camera_module::ROOM_CAMERA_MODULE_TITLE;
use crate::gui::panels::panel_manager::is_mouse_over_panel;
use crate::gui::inspector::player_module::PlayerModule;
use crate::editor_global::push_command;
use crate::gui::menu_bar::menu_button;
use crate::commands::room::*;
use crate::gui::gui_constants::*;
use engine_core::prelude::*;
use bishop::prelude::*;

const SCROLL_SPEED: f32 = 5.0;

/// Returns the entity that should be used for component operations.
fn component_target(ecs: &Ecs, entity: Entity) -> Entity {
    if ecs.has::<PlayerProxy>(entity) {
        ecs.get_player_entity().unwrap_or(entity)
    } else {
        entity
    }
}

/// Returns true if this module should use the proxy entity directly.
fn is_proxy_local_module(module_title: &str) -> bool {
    module_title == comp_type_name::<Transform>()
        || module_title == "PlayerModule"
} 

/// The panel that lives on the right‑hand side of the room editor.
pub struct InspectorPanel {
    /// Geometry of the panel.
    rect: Rect,
    /// Currently inspected entity.
    pub target: Option<Entity>,
    /// All sub‑modules that can draw UI.
    modules: Vec<Box<dyn InspectorModule>>,
    /// If true hide normal panel and show only the add‑component UI.
    add_mode: bool,
    /// Component name that the user selected from the menu.
    pending_add: Option<String>,
    /// Rectangles that were drawn this frame and are therefore active.
    active_rects: Vec<Rect>,
    /// Current vertical offset of the scroll‑view.
    scroll_offset: f32,
    /// Ids of the widgets at the top level of the inspector.
    widget_ids: WidgetIds,
}

pub struct WidgetIds {
    pub darkness_slider_id: WidgetId
}

impl InspectorPanel {
    /// Create a fresh panel with the default set of modules.
    pub fn new() -> Self {
        let mut modules: Vec<Box<dyn InspectorModule>> = Vec::new();

        // Wrap each concrete module in a CollapsibleModule
        modules.push(Box::new(
            PlayerModule::default(),
        ));

        // Collect generic modules here, with Name and Transform handled specially
        let mut name_module: Option<Box<dyn InspectorModule>> = None;
        let mut transform_module: Option<Box<dyn InspectorModule>> = None;
        let mut other_modules: Vec<Box<dyn InspectorModule>> = Vec::new();

        for entry in MODULES.iter() {
            let module = (entry.factory)();

            // Check if this is the Name module
            if entry.title == comp_type_name::<Name>() {
                name_module = Some(module);
            } else if entry.title == comp_type_name::<Transform>() {
                transform_module = Some(module);
            } else {
                other_modules.push(module);
            }
        }

        // Add Name module (after player module) if it exists
        if let Some(name_mod) = name_module {
            modules.insert(1, name_mod);
        }

        // Add Transform module (after Name) if it exists
        if let Some(transform_mod) = transform_module {
            modules.insert(2, transform_mod);
        }

        // Add all other generic modules
        modules.extend(other_modules);

        let widget_ids = WidgetIds {
            darkness_slider_id: WidgetId::default(),
        };
        
        Self {
            rect: Rect::new(0., 0., 0., 0.),
            target: None,
            modules,
            add_mode: false,
            pending_add: None,
            active_rects: Vec::new(),
            scroll_offset: 0.0,
            widget_ids,
        }
    }

    /// Called by the editor each frame to place the panel.
    pub fn set_rect(&mut self, rect: Rect) {
        self.rect = rect;
    }

    /// Tell the inspector which entity is currently selected.
    pub fn set_target(&mut self, entity: Option<Entity>) {
        if self.target != entity {
            self.target = entity;
            self.scroll_offset = 0.0; 
        }
    }

    /// Render the panel and any visible sub‑modules
    /// Returns true if 'Create' was pressed.
    pub fn draw(
        &mut self,
        ctx: &mut WgpuContext,
        game_ctx: &mut GameCtxMut
    ) -> bool {
        self.active_rects.clear();
   
        const BTN_MARGIN: f32 = 10.0;

        // When an entity is selected we show “Remove” and “Add Component”
        if let Some(entity) = self.target {
            if Controls::copy(ctx) {
                copy_entity(game_ctx.ecs, entity);
            }

            // Labels
            let remove_label = "Remove";
            let add_label = "Add Component";

            // Measure text to obtain proper button widths
            let txt_remove = measure_text(ctx, remove_label, HEADER_FONT_SIZE_20);
            let txt_add = measure_text(ctx, add_label, HEADER_FONT_SIZE_20);
            let btn_w_remove = txt_remove.width + WIDGET_PADDING;
            let btn_w_add = txt_add.width + WIDGET_PADDING;

            // Compute left‑most X so the pair stays inside the screen
            let total_w = btn_w_remove + btn_w_add + WIDGET_SPACING;
            let x_start = ctx.screen_width() - INSET - total_w;

            // Add Component button
            let add_rect = self.register_rect(Rect::new(
                x_start + btn_w_remove + WIDGET_SPACING,
                INSET,
                btn_w_add,
                BTN_HEIGHT,
            ));

            // Draw the drop‑down menu when in add mode
            if self.add_mode {
                self.draw_add_component_menu(ctx, add_rect, game_ctx.ecs);
            }

            // Normal inspector UI (hidden while add_mode is true)
            if !self.add_mode {
                // Compute the top offset for the panel
                let top_offset = add_rect.y + BTN_HEIGHT + INSET;

                // Reduce the height so the panel still fits
                let inner = Rect::new(
                    self.rect.x,
                    top_offset,
                    self.rect.w - INSET,
                    self.rect.h - (top_offset - self.rect.y) - INSET,
                );

                // Background
                ctx.draw_rectangle(
                    inner.x,
                    inner.y,
                    inner.w,
                    inner.h,
                    Color::new(0., 0., 0., 0.6),
                );

                let total_content_h = self.total_content_height(game_ctx.ecs, entity);

                if inner.contains(ctx.mouse_position().into()) && !is_mouse_over_dropdown_list(ctx) {
                    let (_, dy) = ctx.mouse_wheel();
                    if dy != 0.0 {
                        let max_offset = (total_content_h - inner.h).max(0.0);
                        self.scroll_offset = (self.scroll_offset - dy * SCROLL_SPEED)
                            .clamp(0.0, max_offset);
                    }
                }

                // Render modules inside the scroll‑view
                let mut y = inner.y + INSET - self.scroll_offset;
                let blocked = self.is_blocked(ctx);
                let comp_target = component_target(game_ctx.ecs, entity);
                for module in &mut self.modules {
                    // Transform uses the proxy directly, others use Player
                    let module_entity = if is_proxy_local_module(module.title()) {
                        entity
                    } else {
                        comp_target
                    };

                    if module.visible(game_ctx.ecs, module_entity) {
                        let h = module.height();

                        // Only draw when the module intersects the visible area
                        if y + h > inner.y && y < inner.y + inner.h {
                            let sub_rect = Rect::new(inner.x + INSET, y, inner.w - INSET * 2.0, h);
                            module.draw(ctx, blocked, sub_rect, game_ctx, module_entity);
                        }

                        y += h + WIDGET_SPACING;
                    }
                }

                // Render deferred dropdown lists on top of modules
                flush_dropdown_lists(ctx);

                // Scroll bar
                if total_content_h > inner.h {
                    // Height of the thumb proportional to the visible fraction
                    let thumb_h = inner.h * inner.h / total_content_h;
                    // Position of the thumb inside the panel
                    let thumb_y = inner.y + (self.scroll_offset / total_content_h) * inner.h;
                    // Draw a simple grey bar on the right edge of the panel
                    ctx.draw_rectangle(
                        inner.x + inner.w - 6.0,
                        thumb_y,
                        4.0,
                        thumb_h,
                        Color::new(0.7, 0.7, 0.7, 0.8),
                    );
                }

                // Cover modules overflowing the top/bottom
                self.draw_overflow_covers(ctx, inner);

                // Outline 
                ctx.draw_rectangle_lines(inner.x, inner.y, inner.w, inner.h, 2., Color::WHITE);
            }
            
            // Draw buttons at the top after the covers
            // Add entity
            if menu_button(ctx, add_rect, add_label, false) 
            && self.can_show_any_component(game_ctx.ecs) {
                self.add_mode = !self.add_mode;
            }

            // Remove button
            // Don't show remove for player entity
            if !(game_ctx.ecs.get_store::<Player>().contains(entity)) {
                let remove_rect = self.register_rect(Rect::new(x_start, INSET, btn_w_remove, BTN_HEIGHT));

                if menu_button(ctx, remove_rect, remove_label, false) || Controls::delete(ctx) && !input_is_focused() {
                    let room_id = game_ctx.cur_world.current_room_id.unwrap_or_default();
                    let command = DeleteEntityCmd {
                        entity,
                        room_id,
                        saved: None,
                    };
                    push_command(Box::new(command));

                    self.target = None;
                    self.add_mode = false;
                    return false;
                }
            }
        } else {
            // No entity selected
            let create_label = "+ Entity";
            let txt_create = measure_text(ctx, create_label, HEADER_FONT_SIZE_20);
            let create_btn = Rect::new(
                self.rect.x + self.rect.w - txt_create.width - BTN_MARGIN - (WIDGET_PADDING * 2.0),
                self.rect.y + BTN_MARGIN,
                txt_create.width + WIDGET_PADDING * 2.0,
                BTN_HEIGHT,
            );

            let add_cam_label = "+ Camera";
            let txt_cam = measure_text(ctx, add_cam_label, HEADER_FONT_SIZE_20);
            let cam_btn_w = txt_cam.width + WIDGET_PADDING * 2.0;
            let cam_btn = Rect::new(
                create_btn.x - WIDGET_SPACING - cam_btn_w,
                create_btn.y,
                cam_btn_w,
                BTN_HEIGHT,
            );

            if menu_button(ctx, cam_btn, add_cam_label, false) {
                // Create a new RoomCamera entity that belongs to the current room
                let ecs = &mut game_ctx.ecs;
                let cur_room = game_ctx.cur_world.current_room().unwrap();
                cur_room.create_room_camera(ecs, cur_room.id, game_ctx.cur_world.grid_size);
            }

            let cur_room = game_ctx.cur_world.current_room_mut().unwrap();

            // Darkness slider
            let slider_width = 150.0;
            let slider_rect = self.register_rect(Rect::new(
                create_btn.x + create_btn.w - slider_width,
                create_btn.y + BTN_HEIGHT + 20.0,
                slider_width,
                BTN_HEIGHT,
            ));

            let (new_val, state) = gui_slider(
                ctx,
                self.widget_ids.darkness_slider_id,
                slider_rect,
                0.0,
                1.0,
                cur_room.darkness,
            );

            if !matches!(state, SliderState::Unchanged) {
                cur_room.darkness = new_val.clamp(0.0, 1.0);
            }

            let txt_val = format!("{:.2}", cur_room.darkness);
            let txt_measure = measure_text(ctx, &txt_val, DEFAULT_FONT_SIZE_16);
            let txt_x = slider_rect.x - txt_measure.width - WIDGET_SPACING;
            let txt_y = slider_rect.y + 20.;
            ctx.draw_text(&txt_val, txt_x, txt_y, 20.0, Color::WHITE);

            return menu_button(ctx, create_btn, create_label, false);
        }

        // Process pending component addition
        if let (Some(name), Some(entity)) = (self.pending_add.take(), self.target) {
            // Add components to Player entity when proxy is selected
            let target = component_target(game_ctx.ecs, entity);
            if let Some(reg) = COMPONENTS.iter().find(|r| r.type_name == name) {
                (reg.factory)(game_ctx.ecs, target);
            } else {
                onscreen_error!("Component `{}` not found in registry", name);
            }
        }
        false
    }

    fn is_blocked(&self, ctx: &mut WgpuContext) -> bool {
        is_mouse_over_panel(ctx)
    }

    /// Draw the drop‑down list that appears under the Add Component button
    fn draw_add_component_menu(
        &mut self,
        ctx: &mut WgpuContext, 
        button_rect: Rect, 
        ecs: &mut Ecs
    ) {
        let entity = match self.target {
            Some(e) => e,
            None => return,
        };

        // For proxies, add components to the Player entity
        let comp_target = component_target(ecs, entity);

        // Collect the components that can be added
        let mut shown: Vec<&ComponentRegistry> = Vec::new();

        for entry in MODULES.iter() {
            let type_name = entry.title;
            // Room cameras must be created separately
            if type_name == ROOM_CAMERA_MODULE_TITLE {
                continue;
            }
            // Transform is local to proxy, skip it in add menu for proxies
            if is_proxy_local_module(type_name) && ecs.has::<PlayerProxy>(entity) {
                continue;
            }
            if let Some(reg) = COMPONENTS.iter().find(|r| r.type_name == type_name) {
                if !entity_has_component(ecs, comp_target, reg) {
                    shown.push(reg);
                }
            } else {
                onscreen_error!("Module `{}` has no ComponentReg entry", type_name);
            }
        }
        
        // Close the menu if nothing to show
        if shown.is_empty() {
            self.add_mode = false;
            return;
        }

        const ENTRY_H: f32 = 30.0;
        const DEFAULT_MENU_W: f32 = 200.0;
        const MIN_INSET: f32 = 10.0;

        // Determine width
        let mut needed_w = DEFAULT_MENU_W;
        for reg in &shown {
            let txt = measure_text(ctx, reg.type_name, DEFAULT_FONT_SIZE_16);
            let w = txt.width + 20.0;
            if w > needed_w {
                needed_w = w;
            }
        }

        // Clamp width to usable screen area
        let max_w = ctx.screen_width() - 2.0 * MIN_INSET;
        let menu_w = needed_w.min(max_w);
        
        // Height depends on number of entries
        let menu_h = (shown.len() as f32) * ENTRY_H + 10.0;

        // Horizontal position
        let mut menu_x = button_rect.x;
        if menu_x + menu_w > ctx.screen_width() - MIN_INSET {
            menu_x = ctx.screen_width() - MIN_INSET - menu_w;
        }
        if menu_x < MIN_INSET {
            menu_x = MIN_INSET;
        }
        // Vertical position
        let menu_y = button_rect.y + button_rect.h + MIN_INSET;
        let menu_rect = self.register_rect(Rect::new(menu_x, menu_y, menu_w, menu_h));

        // Background & border
        ctx.draw_rectangle(
            menu_rect.x,
            menu_rect.y,
            menu_rect.w,
            menu_rect.h,
            Color::new(0.0, 0.0, 0.0, 0.8),
        );
        
        ctx.draw_rectangle_lines(menu_rect.x, menu_rect.y, menu_rect.w, menu_rect.h, 2.0, Color::WHITE);

        // Entries
        for (idx, reg) in shown.iter().enumerate() {
            let entry_rect = Rect::new(
                menu_rect.x + 5.0,
                menu_rect.y + 5.0 + idx as f32 * ENTRY_H,
                menu_rect.w - 10.0,
                25.0,
            );
            if Button::new(entry_rect, reg.type_name).show(ctx) {
                self.pending_add = Some(reg.type_name.to_string());
                self.add_mode = false;
            }
        }
    }

    /// Returns true if the currently selected entity can receive at least one
    /// component that is not already present
    fn can_show_any_component(&self, ecs: &Ecs) -> bool {
        let entity = match self.target {
            Some(e) => e,
            None => return false,
        };
        let comp_target = component_target(ecs, entity);
        let is_spawn_point = ecs.has::<PlayerProxy>(entity);
        for entry in MODULES.iter() {
            let type_name = entry.title;
            // Skip Transform for proxies
            if is_proxy_local_module(type_name) && is_spawn_point {
                continue;
            }
            if let Some(reg) = COMPONENTS.iter().find(|r| r.type_name == type_name) {
                if !entity_has_component(ecs, comp_target, reg) {
                    return true;
                }
            }
        }
        false
    }

    #[inline]
    fn register_rect(&mut self, rect: Rect) -> Rect {
        self.active_rects.push(rect);
        rect
    }

    pub fn is_mouse_over(&self, ctx: &WgpuContext) -> bool {
        let mouse_screen: Vec2 = ctx.mouse_position().into();
        self.active_rects.iter().any(|r| r.contains(mouse_screen))
        || (self.rect.contains(mouse_screen) && self.target.is_some())
    }

    fn total_content_height(&self, ecs: &Ecs, entity: Entity) -> f32 {
        let mut total_content_h = 0.0;
        let comp_target = component_target(ecs, entity);
        for module in &self.modules {
            let module_entity = if is_proxy_local_module(module.title()) {
                entity
            } else {
                comp_target
            };
            if module.visible(ecs, module_entity) {
                total_content_h += module.height() + WIDGET_SPACING;
            }
        }
        // Remove the trailing spacing that we added after the last module
        if total_content_h > 0.0 {
            total_content_h -= WIDGET_SPACING;
        }

        total_content_h += INSET * 2.0; // Top and bottom inset

        total_content_h
    }

    /// Draw the four solid‑grey mask rectangles which hide anything 
    /// that scrolls outside the visible inspector area.
    fn draw_overflow_covers(&self, ctx: &mut WgpuContext, inner: Rect) {
        // Top cover
        ctx.draw_rectangle(
            self.rect.x,
            self.rect.y,
            self.rect.w,
            inner.y - self.rect.y,
            PANEL_COLOR,
        );

        // Bottom cover
        let inner_bottom = inner.y + inner.h;
        let panel_bottom = self.rect.y + self.rect.h;

        ctx.draw_rectangle(
            self.rect.x,
            inner_bottom,
            self.rect.w,
            panel_bottom - inner_bottom,
            PANEL_COLOR,
        );
        
        // Left strip
        ctx.draw_rectangle(
            self.rect.x - INSET,
            self.rect.y,
            INSET,
            self.rect.h,
            PANEL_COLOR,
        );
        
        // Right strip
        let inner_right = inner.x + inner.w;
        let panel_right = self.rect.x + self.rect.w;
        ctx.draw_rectangle(
            inner_right,
            self.rect.y,
            panel_right - inner_right,
            self.rect.h,
            PANEL_COLOR,
        );
    }
}

/// Utility function used by both the panel and the menu
fn entity_has_component(
    ecs: &Ecs,
    entity: Entity,
    reg: &ComponentRegistry,
) -> bool {
    (reg.has)(ecs, entity)
}

