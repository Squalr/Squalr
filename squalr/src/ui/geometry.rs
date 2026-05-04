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

#[cfg(test)]
mod tests {
    use super::{safe_clamp_f32, safe_clamp_ord};

    #[test]
    fn safe_clamp_f32_clamps_with_ordered_bounds() {
        assert_eq!(safe_clamp_f32(5.0, 0.0, 4.0), 4.0);
        assert_eq!(safe_clamp_f32(-1.0, 0.0, 4.0), 0.0);
        assert_eq!(safe_clamp_f32(2.0, 0.0, 4.0), 2.0);
    }

    #[test]
    fn safe_clamp_f32_collapses_to_maximum_when_bounds_are_inverted() {
        assert_eq!(safe_clamp_f32(150.0, 200.0, 100.0), 100.0);
    }

    #[test]
    fn safe_clamp_f32_handles_nan_without_panicking() {
        assert_eq!(safe_clamp_f32(f32::NAN, 2.0, 8.0), 2.0);
        assert_eq!(safe_clamp_f32(5.0, f32::NAN, 8.0), 5.0);
        assert_eq!(safe_clamp_f32(12.0, f32::NAN, 8.0), 8.0);
        assert_eq!(safe_clamp_f32(5.0, 2.0, f32::NAN), 5.0);
    }

    #[test]
    fn safe_clamp_ord_collapses_to_maximum_when_bounds_are_inverted() {
        assert_eq!(safe_clamp_ord(15, 20, 10), 10);
    }
}
