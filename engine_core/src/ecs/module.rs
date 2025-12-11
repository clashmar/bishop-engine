// engine_core/src/ecs/module.rs
use crate::game::game::GameCtxMut;
use crate::ui::text::*;
use crate::ui::widgets::*;
use macroquad::prelude::*;
use crate::ecs::world_ecs::WorldEcs;
use crate::ecs::entity::Entity;

/// Every inspector sub‑module implements this trait.
pub trait InspectorModule {
    /// Return true when the module should be shown for the given entity.
    fn visible(&self, ecs: &WorldEcs, entity: Entity) -> bool;

    // TODO: Make this async
    /// Draw the UI for the module inside the supplied rectangle.
    fn draw(
        &mut self,
        rect: Rect,
        game_ctx: &mut GameCtxMut,
        entity: Entity,
    );

    /// Height in screen pixels that the module occupies when it is
    /// expanded.
    fn height(&self) -> f32 {
        80.0
    }

    /// Title that appears in the collapsible header. Uses the
    /// Rust type name by default and can be overriden.
    fn title(&self) -> &str {
        std::any::type_name::<Self>()
            .rsplit("::")
            .next()
            .unwrap_or("Module")
    }

    /// Return true if the module should get a “Remove” button in the header.
    /// Default is false.
    fn removable(&self) -> bool { false }

    /// Called when the user clicks the remove component button.
    /// Default implementation does nothing.
    fn remove(&mut self, _ecs: &mut WorldEcs, _entity: Entity) {}
}

/// Generic wrapper that adds a collapsible header around any concrete
/// `InspectorModule`.  It stores the `expanded` flag, draws the “‑/＋” button,
/// and forwards the actual drawing to the inner module when expanded.
pub struct CollapsibleModule<T: InspectorModule> {
    inner: T,
    expanded: bool,
    /// Optional custom title. If `None`, ask the inner module for its
    /// `title()` implementation.
    custom_title: Option<String>,
}

impl<T: InspectorModule> CollapsibleModule<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            expanded: true, // start opened   
            custom_title: None,
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.custom_title = Some(title.into());
        self
    }

    fn title(&self) -> &str {
        if let Some(ref t) = self.custom_title {
            t
        } else {
            self.inner.title()
        }
    }

    /// The clickable area that contains the “‑/＋” button, title and remove button.
    const HEADER_HEIGHT: f32 = 24.0;
}

impl<T: InspectorModule> InspectorModule for CollapsibleModule<T> {
    fn visible(&self, ecs: &WorldEcs, entity: Entity) -> bool {
        self.inner.visible(ecs, entity)
    }

    fn draw(
        &mut self,
        rect: Rect,
        game_ctx: &mut GameCtxMut,
        entity: Entity,
    ) {
        let world_ecs = &mut game_ctx.cur_world_ecs;

        // Background for the header
        draw_rectangle(rect.x, rect.y, rect.w, Self::HEADER_HEIGHT, Color::new(0., 0., 0., 0.4));
        draw_text_ui(
            self.title(),
            rect.x + 28.0,
            rect.y + 18.0,
            DEFAULT_FONT_SIZE_16,
            WHITE,
        );

        // Toggle button (‑ when open, ＋ when closed)
        let btn = Rect::new(rect.x + 4.0, rect.y + 4.0, 16.0, 16.0);
        let symbol = if self.expanded { "-" } else { "+" };
        if gui_button_y_offset(btn, symbol, vec2(-0.3, 1.5)) {
            self.expanded = !self.expanded;
        }

        // Remove component
        if self.inner.removable() {
            const BTN_W: f32 = 20.0;
            const BTN_H: f32 = 20.0;
            // Right‑aligned, vertically centred in the header
            let btn_rect = Rect::new(
                rect.x + rect.w - BTN_W - 4.0,
                rect.y + (Self::HEADER_HEIGHT - BTN_H) / 2.0,
                BTN_W,
                BTN_H,
            );
            if gui_button(btn_rect, "x") {



                self.inner.remove(world_ecs, entity);
                return; // Don't draw the rest of the module
            }
        }

        // Body, when expanded
        if self.expanded {
            // Give the inner module a little padding inside the panel
            let body_rect = Rect::new(
                rect.x + 4.0,
                rect.y + Self::HEADER_HEIGHT + 4.0,
                rect.w - 8.0,
                rect.h - Self::HEADER_HEIGHT - 8.0,
            );
            self.inner.draw(body_rect, game_ctx, entity);
        }
    }

    fn height(&self) -> f32 {
        if self.expanded {
            // Full height (header + inner module's height)
            Self::HEADER_HEIGHT + self.inner.height()
        } else {
            Self::HEADER_HEIGHT
        }
    }

    fn title(&self) -> &str {
        self.title()
    }
}