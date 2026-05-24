#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListNavigationDirection {
    Up,
    Down,
}

pub fn resolve_next_index(
    current_index: Option<usize>,
    item_count: usize,
    direction: ListNavigationDirection,
) -> Option<usize> {
    if item_count == 0 {
        return None;
    }

    let Some(current_index) = current_index else {
        return Some(match direction {
            ListNavigationDirection::Up => item_count.saturating_sub(1),
            ListNavigationDirection::Down => 0,
        });
    };

    let next_index = match direction {
        ListNavigationDirection::Up => current_index.saturating_sub(1),
        ListNavigationDirection::Down => current_index.saturating_add(1),
    };

    Some(next_index.min(item_count.saturating_sub(1)))
}
