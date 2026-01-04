use crate::gui::panels::generic_panel::PanelDefinition;
use crate::room::room_editor::RoomEditor;
use crate::gui::gui_constants::*;
use crate::ecs::component::Name;
use crate::ecs::entity::*;
use crate::ecs::ecs::Ecs;
use crate::Editor;
use engine_core::ui::text::draw_text_ui;
use engine_core::ecs::entity::Entity;
use engine_core::ecs::component::*;
use engine_core::ui::widgets::*;
use std::collections::HashSet;
use macroquad::prelude::*;

const ROW_HEIGHT: f32 = 22.0;
const HEADER_HEIGHT: f32 = 18.0;
const ADD_BUTTON_HEIGHT: f32 = 26.0;
const SCROLL_SPEED: f32 = 24.0;
const SCROLLBAR_W: f32 = 6.0;
const TOP_PADDING: f32 = 8.0;
const BOTTOM_PADDING: f32 = 8.0;

pub struct HierarchyPanel {
    expanded: HashSet<Entity>,
    dragging: Option<Entity>,
    drag_offset: Vec2,
    scroll_y: f32,
}

impl HierarchyPanel {
    pub fn new() -> Self {
        Self {
            expanded: HashSet::new(),
            dragging: None,
            drag_offset: Vec2::ZERO,
            scroll_y: 0.0,
        }
    }
}

pub const HIERARCHY_PANEL: &'static str = "Hierarchy";

impl PanelDefinition for HierarchyPanel {
    fn title(&self) -> &'static str {
        HIERARCHY_PANEL
    }

    fn default_rect(&self) -> Rect {
        Rect::new(20., 60., 260., 400.)
    }

    fn draw(&mut self, rect: Rect, editor: &mut Editor) {
        let mouse: Vec2 = mouse_position().into();

        // Scroll input
        if rect.contains(mouse) {
            let (_, wheel_y) = mouse_wheel();
            self.scroll_y += wheel_y * SCROLL_SPEED;
        }

        let ecs = &mut editor.game.ecs;
        let room_editor = &mut editor.room_editor;

        let global_entities = {
            let store = ecs.get_store::<Global>();
            let all: Vec<Entity> = store.data.keys().copied().collect();
            get_root_entities(ecs, &all)
        };
        let room_entities = {
            let store = ecs.get_store::<CurrentRoom>();
            let all: Vec<Entity> = store.data.keys().copied().collect();
            get_root_entities(ecs, &all)
        };

        // Layout pass 
        let mut layout_y = 0.0;

        layout_y += TOP_PADDING;                                  
        layout_y += ADD_BUTTON_HEIGHT + SPACING * 2.0;             
        layout_y += HEADER_HEIGHT;      

        for e in &global_entities {
            layout_entity_tree(*e, &mut layout_y, &self.expanded, ecs);
        }
        layout_y += 10.0;                                       
        layout_y += HEADER_HEIGHT;  

        for e in &room_entities {
            layout_entity_tree(*e, &mut layout_y, &self.expanded, ecs);
        }
        layout_y += BOTTOM_PADDING;                             

        let content_height = layout_y;
        let scroll_range = (content_height - rect.h).max(0.0);
        self.scroll_y = self.scroll_y.clamp(-scroll_range, 0.0);

        // Draw pass 
        let mut y = rect.y + self.scroll_y + TOP_PADDING;

        // Add global button
        let btn_w = inner_width(rect, scroll_range);
        draw_block(
            Rect::new(rect.x + 6., y, btn_w, ADD_BUTTON_HEIGHT),
            rect,
            || {
                if gui_button(
                    Rect::new(rect.x + 6., y, btn_w, ADD_BUTTON_HEIGHT),
                    "+ Global",
                    false,
                ) {
                    ecs.create_entity()
                        .with(Global::default())
                        .with(Name("Global Entity".into()));
                }
            },
        );
        y += ADD_BUTTON_HEIGHT + SPACING * 2.0;

        // Global header
        draw_block(
            Rect::new(rect.x + 6., y, inner_width(rect, scroll_range), HEADER_HEIGHT),
            rect,
            || {
                draw_text_ui("Global", rect.x + 6., y + 14., 14., GRAY);
            },
        );
        y += HEADER_HEIGHT;

        // Global entities
        for entity in global_entities {
            draw_entity_tree(
                entity,
                0,
                rect,
                &mut y,
                &mut self.expanded,
                &mut self.dragging,
                &mut self.drag_offset,
                room_editor,
                ecs,
                scroll_range,
            );
        }

        y += 10.0;

        // Room header
        draw_block(
            Rect::new(rect.x + 6., y, inner_width(rect, scroll_range), HEADER_HEIGHT),
            rect,
            || {
                draw_text_ui("Room", rect.x + 6., y + 14., 14., GRAY);
            },
        );
        y += HEADER_HEIGHT;

        // Room entities
        for entity in room_entities {
            draw_entity_tree(
                entity,
                0,
                rect,
                &mut y,
                &mut self.expanded,
                &mut self.dragging,
                &mut self.drag_offset,
                room_editor,
                ecs,
                scroll_range,
            );
        }

        // Scrollbar
        if scroll_range > 0.0 {
            let ratio = rect.h / content_height;
            let bar_h = rect.h * ratio;
            let t = (-self.scroll_y) / scroll_range;
            let bar_x = rect.x + rect.w - SCROLLBAR_W - 2.0;
            let bar_y = rect.y + t * (rect.h - bar_h);
            draw_rectangle(
                bar_x,
                rect.y,
                SCROLLBAR_W,
                rect.h,
                Color::new(0.15, 0.15, 0.15, 0.6),
            );
            draw_rectangle(
                bar_x,
                bar_y,
                SCROLLBAR_W,
                bar_h,
                Color::new(0.7, 0.7, 0.7, 0.9),
            );
        }

        // Drag ghost
        if let Some(dragged) = self.dragging {
            let (mx, my) = mouse_position();
            let name = get_entity_name(ecs, dragged);
            draw_rectangle(
                mx - self.drag_offset.x,
                my - self.drag_offset.y,
                150.0,
                ROW_HEIGHT,
                Color::new(0.3, 0.5, 0.7, 0.5),
            );
            draw_text_ui(
                &name,
                mx - self.drag_offset.x + 4.0,
                my - self.drag_offset.y + 16.0,
                14.0,
                WHITE,
            );
            if is_mouse_button_released(MouseButton::Left) {
                self.dragging = None;
            }
        }
    }
}

fn layout_entity_tree(
    entity: Entity,
    y: &mut f32,
    expanded: &HashSet<Entity>,
    ecs: &Ecs,
) {
    *y += ROW_HEIGHT;
    if expanded.contains(&entity) && has_children(ecs, entity) {
        for child in get_children(ecs, entity) {
            layout_entity_tree(child, y, expanded, ecs);
        }
    }
}

fn draw_entity_tree(
    entity: Entity,
    depth: usize,
    panel_rect: Rect,
    y: &mut f32,
    expanded: &mut HashSet<Entity>,
    dragging: &mut Option<Entity>,
    drag_offset: &mut Vec2,
    room_editor: &mut RoomEditor,
    ecs: &mut Ecs,
    scroll_range: f32,
) {
    // Width that respects the scrollbar
    let usable_w = inner_width(panel_rect, scroll_range);
    let indent = depth as f32 * 16.0;
    let row_rect = Rect::new(
        panel_rect.x + 6. + indent,
        *y,
        usable_w - indent,
        ROW_HEIGHT,
    );

    draw_block(row_rect, panel_rect, || {
        let has_children = has_children(ecs, entity);
        let is_expanded = expanded.contains(&entity);
        let mouse: Vec2 = mouse_position().into();
        let mouse_over = row_rect.contains(mouse);

        // Selection highlight
        if room_editor.selected_entity == Some(entity) {
            draw_rectangle(
                row_rect.x,
                row_rect.y,
                row_rect.w,
                row_rect.h,
                Color::new(0.25, 0.45, 0.85, 0.35),
            );
        }

        // Expand/collapse buttons
        if has_children {
            let btn = Rect::new(row_rect.x, row_rect.y, 14.0, ROW_HEIGHT);
            let symbol = if is_expanded { "-" } else { "+" };
            if gui_button_plain_hover(btn, symbol, WHITE, GRAY, false) {
                if is_expanded {
                    expanded.remove(&entity);
                } else {
                    expanded.insert(entity);
                }
            }
        }

        // Selection
        if mouse_over && is_mouse_button_pressed(MouseButton::Left) && dragging.is_none() {
            room_editor.set_selected_entity(Some(entity));
        }

        // Start drag
        if mouse_over && is_mouse_button_down(MouseButton::Left) && dragging.is_none() {
            *dragging = Some(entity);
            *drag_offset = mouse - row_rect.point();
        }

        // Drop target to parent
        if let Some(dragged) = *dragging {
            if dragged != entity && mouse_over && !is_ancestor(ecs, dragged, entity) {
                draw_rectangle(
                    row_rect.x,
                    row_rect.y,
                    row_rect.w,
                    row_rect.h,
                    Color::new(0.3, 0.7, 0.3, 0.3),
                );
                if is_mouse_button_released(MouseButton::Left) {
                    set_parent(ecs, dragged, entity);
                    expanded.insert(entity);
                    *dragging = None;
                }
            }
        }

        // Entity name
        draw_text_ui(
            &get_entity_name(ecs, entity),
            row_rect.x + 18.0,
            row_rect.y + 16.0,
            14.0,
            WHITE,
        );
    });

    *y += ROW_HEIGHT;

    // Recursively draw children
    if expanded.contains(&entity) && has_children(ecs, entity) {
        for child in get_children(ecs, entity) {
            draw_entity_tree(
                child,
                depth + 1,
                panel_rect,
                y,
                expanded,
                dragging,
                drag_offset,
                room_editor,
                ecs,
                scroll_range,
            );
        }
    }

    // Unparent by dragging outside panel
    if let Some(dragged) = *dragging {
        if dragged == entity {
            let mouse: Vec2 = mouse_position().into();
            if !panel_rect.contains(mouse) && is_mouse_button_released(MouseButton::Left) {
                remove_parent(ecs, dragged);
                *dragging = None;
            }
        }
    }
}

fn inner_width(panel: Rect, scroll_range: f32) -> f32 {
    if scroll_range > 0.0 {
        panel.w - 12.0 - SCROLLBAR_W 
    } else {
        panel.w - 12.0
    }
}

fn draw_block<F: FnOnce()>(block: Rect, clip: Rect, f: F) {
    if block.y >= clip.y && block.y + block.h <= clip.y + clip.h {
        f();
    }
}

fn get_entity_name(ecs: &Ecs, entity: Entity) -> String {
    ecs.get::<Name>(entity)
        .map(|n| n.to_string())
        .unwrap_or_else(|| format!("{:?}", entity))
}