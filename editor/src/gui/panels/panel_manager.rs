// editor/src/gui/panels/panel_manager.rs
use crate::gui::panels::generic_panel::*;
use crate::editor::EditorMode;
use crate::Editor;
use std::collections::HashMap;

pub struct PanelManager {
    panels: HashMap<PanelId, GenericPanel>,
    panel_modes: HashMap<PanelId, Vec<EditorMode>>,
}

impl PanelManager {
    pub fn new() -> Self {
        Self {
            panels: HashMap::new(),
            panel_modes: HashMap::new(),
        }
    }

    pub fn register(
        &mut self,
        panel: GenericPanel,
        modes: Vec<EditorMode>,
    ) {
        self.panel_modes.insert(panel.title, modes);
        self.panels.insert(panel.title, panel);
    }

    pub fn draw(&mut self, editor_mode: EditorMode, editor: &mut Editor) {
        for (id, panel) in self.panels.iter_mut() {
            if self.panel_modes[id].contains(&editor_mode) {
                panel.update_and_draw(editor);
            }
        }
    }

    pub fn toggle(&mut self, id: PanelId) {
        if let Some(p) = self.panels.get_mut(id) {
            p.visible = !p.visible;
        }
    }
}
