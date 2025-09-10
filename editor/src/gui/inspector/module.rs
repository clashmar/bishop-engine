// editor/src/gui/inspector/module.rs
use crate::gui::*;
use macroquad::prelude::*;
use engine_core::assets::asset_manager::AssetManager;
use engine_core::ecs::world_ecs::WorldEcs;
use engine_core::ecs::entity::Entity;

/// Every inspector sub‑module implements this trait.
pub trait InspectorModule {
    /// Return true when the module should be shown for the given entity.
    fn visible(&self, ecs: &WorldEcs, entity: Entity) -> bool;

    /// Draw the UI for the module inside the supplied rectangle.
    fn draw(
        &mut self,
        rect: Rect,
        assets: &mut AssetManager,
        ecs: &mut WorldEcs,
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

    /// The clickable area that contains the “‑/＋” button and the title.
    const HEADER_HEIGHT: f32 = 24.0;
}

impl<T: InspectorModule> InspectorModule for CollapsibleModule<T> {
    fn visible(&self, ecs: &WorldEcs, entity: Entity) -> bool {
        self.inner.visible(ecs, entity)
    }

    fn draw(
        &mut self,
        rect: Rect,
        assets: &mut AssetManager,
        ecs: &mut WorldEcs,
        entity: Entity,
    ) {
        // Background for the header
        draw_rectangle(rect.x, rect.y, rect.w, Self::HEADER_HEIGHT, Color::new(0., 0., 0., 0.4));
        draw_text(
            self.title(),
            rect.x + 28.0,
            rect.y + 18.0,
            18.0,
            WHITE,
        );

        // Toggle button (‑ when open, ＋ when closed)
        let btn = Rect::new(rect.x + 4.0, rect.y + 4.0, 16.0, 16.0);
        let symbol = if self.expanded { "-" } else { "+" };
        if gui_button(btn, symbol) {
            self.expanded = !self.expanded;
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
            self.inner
                .draw(body_rect, assets, ecs, entity);
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