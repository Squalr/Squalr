#[derive(Debug, Clone)]
pub enum VectorizationSize {
    Vector16,
    Vector32,
    Vector64,
}

impl Default for VectorizationSize {
    fn default() -> Self {
        Self::Vector16
    }
}
