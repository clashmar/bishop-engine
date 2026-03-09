use bishop::Vec2;

/// Linearly interpolates between two positions and rounds to the nearest pixel.
#[inline]
pub fn lerp_rounded(prev_pos: Vec2, current_pos: Vec2, alpha: f32) -> Vec2 {
    (prev_pos * (1.0 - alpha) + current_pos * alpha).round()
}

/// Mitigates erratic dt by smoothing `raw_dt`, initializing from the first sample.
#[inline]
pub fn smooth_dt(smoothed_dt: &mut Option<f32>, raw_dt: f32, alpha: f32) -> f32 {
    let s = smoothed_dt.get_or_insert(raw_dt);
    *s = *s * alpha + raw_dt * (1.0 - alpha);
    *s
}