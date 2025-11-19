// editor/src/gui/menu_bar.rs
use crate::gui::gui_constants::*;
use crate::gui::inspector::modal::is_modal_open;
use std::cell::RefCell;
use std::fmt::{self, Display};
use engine_core::ui::text::*;
use engine_core::ui::widgets::*;
use macroquad::prelude::*;
use strum_macros::EnumIter;

/// Holds the state of the top‑level menu bar.
pub struct MenuBar {
    file_id: WidgetId,
    edit_id: WidgetId,
    title_id: WidgetId,
    pub pending: Option<MenuAction>,
}

#[derive(EnumIter, Clone, Copy, PartialEq, Eq, Debug)]
pub enum MenuAction {
    // Rename Game/World/Room
    Rename,
    // File actions
    NewGame,
    Open,
    Save,
    SaveAs,
    // Edit actions
    Undo,
    Redo,
}

impl MenuAction {
    /// Returns the text that should be shown in dropdowns, lists, etc.
    pub fn ui_label(&self) -> String {
        match self {
            MenuAction::NewGame => "New Game".to_string(),
            MenuAction::Save => "Save".to_string(),
            MenuAction::SaveAs => "Save As".to_string(),
            MenuAction::Undo => "Undo".to_string(),
            MenuAction::Redo => "Redo".to_string(),
            _ => format!("{self:?}"),
        }
    }

    /// Optional platform-specific display string for a shortcut.
    pub fn shortcut(&self) -> Option<&'static str> {
        // Windows / Linux
        #[cfg(any(target_os = "windows", target_os = "linux"))]
        {
            match self {
                MenuAction::Save => Some("^ S"),
                MenuAction::SaveAs => Some("⇧ ^ S"),
                MenuAction::Undo => Some("^ Z"),
                MenuAction::Redo => Some("⇧ ^ Z"),       
                _ => None,
            }
        }

        // macOS
        #[cfg(target_os = "macos")]
        {
            match self {
                MenuAction::Save => Some("^ S"),
                MenuAction::SaveAs => Some("⇧ ^ S"),
                MenuAction::Undo => Some("^ Z"),
                MenuAction::Redo => Some("⇧ ^ Z"),
                _ => None,
            }
        }

        // Fallback
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            None
        }
    }
}

impl fmt::Display for MenuAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ui_label())
    }
}

impl MenuBar {
    pub fn new() -> Self {
        Self {
            title_id: WidgetId::default(),
            file_id: WidgetId::default(),
            edit_id: WidgetId::default(),
            pending: None,
        }
    }

    /// Draw the menu options and return any requested action.
    pub fn draw(&mut self, title: &str) -> Option<MenuAction> {
        // Height of each dropdown item
        const HEIGHT: f32 = 30.0;

        // The panel is already drawn in each sub editor
        let panel_rect = menu_panel_rect(); 

        let mut x = panel_rect.x + PADDING;
        let y = panel_rect.y + PADDING / 2.0;

        let title_rect = Rect::new(
            x,
            y,
            rect_width_for_text(title, HEADER_FONT_SIZE_20),
            HEIGHT,
        );

        let title_actions: Vec<MenuAction> = vec![
            MenuAction::Rename,
        ];

        if let Some(selected) = menu_dropdown(
            self.title_id,
            title_rect,
            title,
            &title_actions,
            |a| a.ui_label(),
            |a| a.shortcut(),
        ) {
            self.pending = Some(selected);
        }

        x += title_rect.w + SPACING;

        // File dropdown
        let file_label = "File";

        let file_rect = Rect::new(
            x, 
            y, 
            rect_width_for_text(file_label, HEADER_FONT_SIZE_20), 
            HEIGHT
        );

        let file_actions: Vec<MenuAction> = vec![
            MenuAction::NewGame,
            MenuAction::Open,
            MenuAction::Save,
            MenuAction::SaveAs,
        ];

        if let Some(selected) = menu_dropdown(
            self.file_id,
            file_rect,
            file_label,
            &file_actions,
            |a| a.ui_label(),
            |a| a.shortcut(),
        ) {
            self.pending = Some(selected);
        }

        x += file_rect.w + SPACING;

        // Edit dropdown
        let edit_label = "Edit";

        let edit_rect = Rect::new(
            x, 
            y, 
            rect_width_for_text(edit_label, HEADER_FONT_SIZE_20), 
            HEIGHT
        );

        let edit_actions: Vec<MenuAction> = vec![
            MenuAction::Undo,
            MenuAction::Redo,
        ];

        if let Some(selected) = menu_dropdown(
            self.edit_id,
            edit_rect,
            edit_label,
            &edit_actions,
            |a| a.ui_label(),
            |a| a.shortcut(),
        ) {
            self.pending = Some(selected);
        }

        // Return the action
        self.pending.take()
    }
}

/// Draws a the panel background for the top menu across the whole width of the screen and returns its `Rect`.
pub fn draw_top_panel_full() -> Rect {
    let rect = menu_panel_rect();
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, PANEL_COLOR);
    rect
}

fn menu_panel_rect() -> Rect {
    Rect::new(0.0, 0.0, screen_width(), MENU_PANEL_HEIGHT)
}

/// Button and dropdown for a menu option. 
fn menu_dropdown<T: Clone + PartialEq + Display>(
    id: WidgetId,
    rect: Rect,
    label: &str,
    options: &[T],
    to_string: impl Fn(&T) -> String,
    shortcut: impl Fn(&T) -> Option<&str>
) -> Option<T> {
    const W_PADDING: f32 = 8.0;
    const DROPDOWN_Y_OFFSET: f32 = 7.5;

    // Load previous state
    let mut state = dropdown_state::get(id);

    let mouse_pos: Vec2 = mouse_position().into();
    let hovered = rect.contains(mouse_pos);

    // Change dropdown on mouse hover (if a dropdown is open)
    if hovered {
        let any_open = DROPDOWN_OPEN.with(|f| *f.borrow());
        if any_open {
            CURRENT_OPEN.with(|c| {
                let current_id = *c.borrow();
                if current_id != Some(id) {
                    // Close the previous dropdown
                    if let Some(prev_id) = current_id {
                        let mut prev_state = dropdown_state::get(prev_id);
                        prev_state.open = false;
                        dropdown_state::set(prev_id, prev_state);
                    }
                    // Open the new one
                    state.open = true;
                    *c.borrow_mut() = Some(id);
                }
            });
        }
    }

    // Dropdown header
    let button_clicked = menu_button(rect, label, state.open);

    if button_clicked {
        // Clicking the button toggles open state
        state.open = !state.open;
        // Update the global currently open dropdown
        if state.open {
            CURRENT_OPEN.with(|c| *c.borrow_mut() = Some(id));
        } else {
            CURRENT_OPEN.with(|c| *c.borrow_mut() = None);
        }
    }

    // Decide whether the list should be open this frame
    let list_is_open = state.open; 
    state.open = list_is_open; // Remember for next frame   

    // Let the editor know a dropdown is open
    let mut any_open = false;
    DROPDOWN_OPEN.with(|f| {
        let was = *f.borrow();
        *f.borrow_mut() = was || list_is_open;
        any_open = *f.borrow();
    });     

    // Compute the widest option
    let mut max_opt_width = 0.0_f32;
    for opt in options.iter() {
        // label width
        let label_w = measure_text_ui(&to_string(opt), DEFAULT_FONT_SIZE_16, 1.0).width;
        // optional shortcut width
        let shortcut_w = shortcut(opt)
            .map(|s| measure_text_ui(s, DEFAULT_FONT_SIZE_16, 1.0).width + SPACING)
            .unwrap_or(0.0);
        let total_w = label_w + shortcut_w;
        if total_w > max_opt_width {
            max_opt_width = total_w;
        }
    }
    
    let list_width = rect.w
    .max(max_opt_width + 2.0 * W_PADDING);

    let rows = options.len();
    let total_height = rect.h * rows as f32;

    // Compute the list rectangle
    let list_rect = Rect::new(
        rect.x,
        rect.y + rect.h + DROPDOWN_Y_OFFSET,
        list_width,
        total_height,
    );

    if list_is_open {
        state.rect = list_rect;             
    }

    // Draw the list and handle selection
    if list_is_open {
        let mouse_pos = mouse_position().into();

        // Background
        draw_rectangle(
            list_rect.x,
            list_rect.y,
            list_rect.w,
            list_rect.h,
            PANEL_COLOR,
        );
        
        for (i, opt) in options.iter().enumerate() {
            // The Y position the entry would have without scrolling
            let entry_y = list_rect.y + i as f32 * rect.h;

            let entry_rect = Rect::new(
                list_rect.x,
                entry_y,
                list_rect.w,
                rect.h,
            );

            let hovered = entry_rect.contains(mouse_pos);
            if hovered && is_mouse_button_pressed(MouseButton::Left) {
                // Close the list and return the chosen value
                state.open = false;
                dropdown_state::set(id, state);
                update_global_dropdown_flag();
                return Some(opt.clone());
            }

            if hovered {
                draw_rectangle(
                    entry_rect.x,
                    entry_rect.y,
                    entry_rect.w,
                    entry_rect.h,
                    Color::new(0.2, 0.2, 0.2, 0.9),
                );
            }
            
            // Action 
            draw_text_ui(
                &to_string(opt),
                entry_rect.x + 5.,
                entry_rect.y + entry_rect.h * 0.7,
                DEFAULT_FONT_SIZE_16,
                BLACK
            );

            // Optional shortcut display
            if let Some(shortcut) = shortcut(opt) {
                let sc_width = measure_text_ui(shortcut, DEFAULT_FONT_SIZE_16, 1.0).width;
                let sc_x = entry_rect.x + entry_rect.w - sc_width - 5.0;
                draw_text_ui(
                    shortcut,
                    sc_x,
                    entry_rect.y + entry_rect.h * 0.7,
                    DEFAULT_FONT_SIZE_16,
                    WHITE,
                );
            }

            // Draw the outline last
            draw_rectangle_lines(
                list_rect.x, 
                list_rect.y, 
                list_rect.w, 
                list_rect.h, 
                2., 
                BLACK
            );
        }
    }

    // Clicking outside closes the dropdown
    let mouse_pos = mouse_position().into();
    if is_mouse_button_pressed(MouseButton::Left)
        && !rect.contains(mouse_pos)
        && !(state.open && state.rect.contains(mouse_pos))
    {
        state.open = false;
        CURRENT_OPEN.with(|c| *c.borrow_mut() = None);
    }

    // Persist the state
    dropdown_state::set(id, state);
    update_global_dropdown_flag();
    None
}

/// Returns true if clicked
pub fn menu_button(
    rect: Rect, 
    label: &str,
    is_dropdown_open: bool,
) -> bool {
    // Text layout
    let txt_y = rect.y + rect.h * 0.7;
    let txt_x = rect.x + PADDING / 2.0;

    let mouse = mouse_position();
    let hovered = rect.contains(vec2(mouse.0, mouse.1));

    if (hovered || is_dropdown_open) && !is_modal_open() {
        draw_rectangle(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            Color::new(0.0, 0.0, 0.0, 0.5),
        );
    }
    
    draw_text_ui(
        label, 
        txt_x, 
        txt_y,
        HEADER_FONT_SIZE_20,
        BLACK
    );

    is_mouse_button_pressed(MouseButton::Left) 
    && hovered
    && !is_modal_open()
}

thread_local! {
    /// Holds the `WidgetId` of the dropdown that is currently open, if any.
    static CURRENT_OPEN: RefCell<Option<WidgetId>> = RefCell::new(None);
}