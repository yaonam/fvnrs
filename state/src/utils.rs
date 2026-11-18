use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as u64
}

pub fn interp(start: f32, end: f32, t: f32) -> f32 {
    start + (end - start) * t
}

/// Interpolates between two floats.
/// Assigns None values as 0.0
pub fn tween_float(start: Option<f32>, end: Option<f32>, t: f32) -> Option<f32> {
    match (start, end) {
        (Some(start), Some(end)) => Some(interp(start, end, t)),
        (Some(start), None) => Some(interp(start, 0.0, t)),
        (None, Some(end)) => Some(interp(0.0, end, t)),
        (None, None) => None,
    }
}

/// Interpolates between two vec2s.
/// Assigns None values as [0.0, 0.0]
pub fn tween_float_vec2(
    start: Option<[f32; 2]>,
    end: Option<[f32; 2]>,
    t: f32,
) -> Option<[f32; 2]> {
    match (start, end) {
        (Some(start), Some(end)) => {
            Some([interp(start[0], end[0], t), interp(start[1], end[1], t)])
        }
        (Some(start), None) => Some([interp(start[0], 0.0, t), interp(start[1], 0.0, t)]),
        (None, Some(end)) => Some([interp(0.0, end[0], t), interp(0.0, end[1], t)]),
        (None, None) => None,
    }
}

/// Merges the two options using the provided function.
/// If `a` is None, defaults to `b`.
pub fn merge_optional_values<T>(
    a_opt: Option<T>,
    b_opt: Option<T>,
    merger: fn(T, T) -> T,
) -> Option<T> {
    match (a_opt, b_opt) {
        (Some(a), Some(b)) => Some(merger(a, b)),
        (a, b) => a.or(b),
    }
}
