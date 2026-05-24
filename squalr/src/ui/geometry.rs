/// Clamps an `f32` without using `f32::clamp`, which panics on invalid bounds.
pub fn safe_clamp_f32(
    value: f32,
    minimum: f32,
    maximum: f32,
) -> f32 {
    match (minimum.is_nan(), maximum.is_nan(), value.is_nan()) {
        (_, _, true) => {
            if minimum.is_nan() {
                return if maximum.is_nan() { 0.0 } else { maximum };
            }

            return minimum;
        }
        (true, true, false) => value,
        (true, false, false) => value.min(maximum),
        (false, true, false) => value.max(minimum),
        (false, false, false) => {
            if minimum > maximum {
                return maximum;
            }

            value.max(minimum).min(maximum)
        }
    }
}

/// Clamps an ordered value without using `Ord::clamp`, which panics on invalid bounds.
pub fn safe_clamp_ord<T>(
    value: T,
    minimum: T,
    maximum: T,
) -> T
where
    T: Copy + Ord,
{
    if minimum > maximum {
        return maximum;
    }

    value.max(minimum).min(maximum)
}
