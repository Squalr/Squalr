use crate::models::docking::hierarchy::types::dock_reparent_direction::DockReparentDirection;
use epaint::{Pos2, Rect, pos2, vec2};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DockDropTarget {
    pub target_window_identifier: String,
    pub direction: DockReparentDirection,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DockDropZone {
    pub drop_target: DockDropTarget,
    pub button_rect: Rect,
    pub preview_rect: Rect,
    pub is_hovered: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DockDragOverlay {
    pub drop_zones: Vec<DockDropZone>,
    pub hovered_drop_target: Option<DockDropTarget>,
}

impl DockDragOverlay {
    pub fn from_window_rectangles(
        target_window_rectangles: &[(String, Rect)],
        pointer_position: Pos2,
    ) -> Self {
        let mut drop_zones = Vec::new();

        for (target_window_identifier, target_window_rect) in target_window_rectangles {
            drop_zones.extend(DockDropZone::build_for_window(target_window_identifier, *target_window_rect));
        }

        let hovered_drop_target = drop_zones
            .iter()
            .find(|dock_drop_zone| dock_drop_zone.button_rect.contains(pointer_position))
            .map(|dock_drop_zone| dock_drop_zone.drop_target.clone());

        if let Some(hovered_drop_target) = hovered_drop_target.as_ref() {
            for dock_drop_zone in drop_zones.iter_mut() {
                dock_drop_zone.is_hovered = dock_drop_zone.drop_target == *hovered_drop_target;
            }
        }

        Self {
            drop_zones,
            hovered_drop_target,
        }
    }
}

impl DockDropZone {
    const DEFAULT_BUTTON_SIDE_PX: f32 = 36.0;
    const MIN_BUTTON_SIDE_PX: f32 = 18.0;
    const BUTTON_SCALE_FACTOR: f32 = 0.18;
    const BUTTON_SPACING_FACTOR: f32 = 0.18;
    const PREVIEW_MARGIN_FACTOR: f32 = 0.04;
    const MIN_PREVIEW_MARGIN_PX: f32 = 6.0;
    const MAX_PREVIEW_MARGIN_PX: f32 = 12.0;

    fn build_for_window(
        target_window_identifier: &str,
        target_window_rect: Rect,
    ) -> Vec<Self> {
        let button_side_px = (target_window_rect.size().min_elem() * Self::BUTTON_SCALE_FACTOR).clamp(Self::MIN_BUTTON_SIDE_PX, Self::DEFAULT_BUTTON_SIDE_PX);
        let button_spacing_px = (button_side_px * Self::BUTTON_SPACING_FACTOR).clamp(4.0, 8.0);
        let target_center = target_window_rect.center();

        [
            (DockReparentDirection::Tab, target_center),
            (
                DockReparentDirection::Left,
                pos2(target_center.x - button_side_px - button_spacing_px, target_center.y),
            ),
            (
                DockReparentDirection::Right,
                pos2(target_center.x + button_side_px + button_spacing_px, target_center.y),
            ),
            (
                DockReparentDirection::Top,
                pos2(target_center.x, target_center.y - button_side_px - button_spacing_px),
            ),
            (
                DockReparentDirection::Bottom,
                pos2(target_center.x, target_center.y + button_side_px + button_spacing_px),
            ),
        ]
        .into_iter()
        .map(|(direction, button_center)| {
            let drop_target = DockDropTarget {
                target_window_identifier: target_window_identifier.to_string(),
                direction,
            };

            Self {
                drop_target: drop_target.clone(),
                button_rect: Rect::from_center_size(button_center, vec2(button_side_px, button_side_px)),
                preview_rect: Self::build_preview_rect(target_window_rect, direction),
                is_hovered: false,
            }
        })
        .collect()
    }

    fn build_preview_rect(
        target_window_rect: Rect,
        direction: DockReparentDirection,
    ) -> Rect {
        let preview_margin_px =
            (target_window_rect.size().min_elem() * Self::PREVIEW_MARGIN_FACTOR).clamp(Self::MIN_PREVIEW_MARGIN_PX, Self::MAX_PREVIEW_MARGIN_PX);
        let preview_rect = match direction {
            DockReparentDirection::Tab => target_window_rect,
            DockReparentDirection::Left => Rect::from_min_max(target_window_rect.min, pos2(target_window_rect.center().x, target_window_rect.max.y)),
            DockReparentDirection::Right => Rect::from_min_max(pos2(target_window_rect.center().x, target_window_rect.min.y), target_window_rect.max),
            DockReparentDirection::Top => Rect::from_min_max(target_window_rect.min, pos2(target_window_rect.max.x, target_window_rect.center().y)),
            DockReparentDirection::Bottom => Rect::from_min_max(pos2(target_window_rect.min.x, target_window_rect.center().y), target_window_rect.max),
        };

        preview_rect.shrink(preview_margin_px)
    }
}

#[cfg(test)]
mod tests {
    use super::{DockDragOverlay, DockDropTarget};
    use crate::models::docking::hierarchy::types::dock_reparent_direction::DockReparentDirection;
    use epaint::{Rect, pos2};

    #[test]
    fn hovered_drop_target_matches_hovered_button() {
        let target_window_rect = Rect::from_min_max(pos2(100.0, 100.0), pos2(400.0, 300.0));
        let initial_overlay = DockDragOverlay::from_window_rectangles(&[("target".to_string(), target_window_rect)], target_window_rect.center());
        let center_drop_zone = initial_overlay
            .drop_zones
            .iter()
            .find(|dock_drop_zone| dock_drop_zone.drop_target.direction == DockReparentDirection::Tab)
            .cloned()
            .expect("expected center drop zone");

        let hovered_overlay = DockDragOverlay::from_window_rectangles(&[("target".to_string(), target_window_rect)], center_drop_zone.button_rect.center());

        assert_eq!(
            hovered_overlay.hovered_drop_target,
            Some(DockDropTarget {
                target_window_identifier: "target".to_string(),
                direction: DockReparentDirection::Tab,
            })
        );
        assert!(
            hovered_overlay
                .drop_zones
                .iter()
                .any(|dock_drop_zone| dock_drop_zone.is_hovered && dock_drop_zone.drop_target.direction == DockReparentDirection::Tab)
        );
    }

    #[test]
    fn split_preview_rectangles_cover_expected_half() {
        let target_window_rect = Rect::from_min_max(pos2(100.0, 100.0), pos2(500.0, 300.0));
        let dock_drag_overlay = DockDragOverlay::from_window_rectangles(&[("target".to_string(), target_window_rect)], target_window_rect.center());

        let left_drop_zone = dock_drag_overlay
            .drop_zones
            .iter()
            .find(|dock_drop_zone| dock_drop_zone.drop_target.direction == DockReparentDirection::Left)
            .expect("expected left drop zone");
        let top_drop_zone = dock_drag_overlay
            .drop_zones
            .iter()
            .find(|dock_drop_zone| dock_drop_zone.drop_target.direction == DockReparentDirection::Top)
            .expect("expected top drop zone");

        assert!(left_drop_zone.preview_rect.max.x <= target_window_rect.center().x);
        assert!(top_drop_zone.preview_rect.max.y <= target_window_rect.center().y);
    }
}
