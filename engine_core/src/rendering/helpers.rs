use bishop::Vec2;

#[inline]
pub fn lerp_floored(prev_pos: Vec2, current_pos: Vec2, alpha: f32) -> Vec2 {
    (prev_pos * (1.0 - alpha) + current_pos * alpha).round()
}