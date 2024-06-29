#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RegionBoundsHandling {
    Exclude,
    Include,
    Resize,
}
