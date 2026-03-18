use bishop::Color;

pub const WIDGET_PADDING: f32 = 10.0;
pub const WIDGET_SPACING: f32 = 10.0;
pub const DEFAULT_FONT_SIZE_16: f32 = 16.0;
pub const HEADER_FONT_SIZE_20: f32 = 20.0;
pub const FIELD_TEXT_SIZE_16: f32 = 16.0;
pub const DEFAULT_FIELD_HEIGHT: f32 = 30.0;
pub const DEFAULT_CHECKBOX_DIMS: f32 = 20.0;

pub const FIELD_TEXT_COLOR: Color = Color::WHITE;
pub const OUTLINE_COLOR: Color = Color::WHITE;
pub const FIELD_BACKGROUND_COLOR: Color = Color::new(0., 0., 0., 1.0);
pub const HOVER_COLOR: Color = Color::new(0.2, 0.2, 0.2, 0.8);
pub const HOVER_COLOR_PLAIN: Color = Color::new(0.2, 0.2, 0.2, 0.8);

pub const HOLD_INITIAL_DELAY: f64 = 0.50;
pub const HOLD_REPEAT_RATE: f64 = 0.05;
pub const PLACEHOLDER_TEXT: &str = "<type here>";
