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

#[cfg(test)]
mod tests {
    use super::{ListNavigationDirection, resolve_next_index};

    #[test]
    fn resolve_next_index_moves_within_bounds() {
        assert_eq!(resolve_next_index(Some(1), 3, ListNavigationDirection::Up), Some(0));
        assert_eq!(resolve_next_index(Some(1), 3, ListNavigationDirection::Down), Some(2));
    }

    #[test]
    fn resolve_next_index_clamps_to_edges() {
        assert_eq!(resolve_next_index(Some(0), 3, ListNavigationDirection::Up), Some(0));
        assert_eq!(resolve_next_index(Some(2), 3, ListNavigationDirection::Down), Some(2));
    }

    #[test]
    fn resolve_next_index_picks_edge_without_current_index() {
        assert_eq!(resolve_next_index(None, 3, ListNavigationDirection::Up), Some(2));
        assert_eq!(resolve_next_index(None, 3, ListNavigationDirection::Down), Some(0));
    }

    #[test]
    fn resolve_next_index_returns_none_for_empty_lists() {
        assert_eq!(resolve_next_index(None, 0, ListNavigationDirection::Down), None);
    }
}
