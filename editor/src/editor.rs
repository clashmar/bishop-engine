use crate::tilemap::tilemap_editor::TileMapEditor;

pub struct EditorState {
    pub tilemap_editor: TileMapEditor,
}

impl EditorState {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            tilemap_editor: TileMapEditor::new(width, height),
        }
    }

    pub fn update(&mut self) {
        self.tilemap_editor.update();
    }

    pub fn draw(&self)  {
        self.tilemap_editor.draw();
    }
}