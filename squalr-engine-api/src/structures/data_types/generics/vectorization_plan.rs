#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VectorizationPlan {
    /// Vector width in bytes (ie 16/32/64).
    pub vector_size_in_bytes: u64,

    /// Stride between candidate element starts (usually memory_alignment as u64).
    pub element_stride_bytes: u64,

    /// Bytes that are valid as *starting positions* (region_size minus trailing bytes).
    pub valid_bytes: u64,

    /// Count of candidate element starts.
    pub element_count: u64,

    /// How many elements fit per vector load (vector_bytes / stride).
    pub elements_per_vector: u64,
}

impl VectorizationPlan {
    pub fn is_valid(&self) -> bool {
        self.elements_per_vector > 0 && self.element_count >= self.elements_per_vector
    }

    pub fn get_vectorizable_iterations(&self) -> u64 {
        if self.elements_per_vector == 0 {
            0
        } else {
            self.element_count / self.elements_per_vector
        }
    }

    pub fn get_vectorizable_element_count(&self) -> u64 {
        self.get_vectorizable_iterations() * self.elements_per_vector
    }

    // Gets the number of elements not
    pub fn get_remainder_elements(&self) -> u64 {
        self.element_count - self.get_vectorizable_element_count()
    }

    /// Tail bytes that remain after full vector iterations, measured in *valid bytes* space.
    pub fn get_remainder_bytes(&self) -> u64 {
        self.valid_bytes - (self.get_vectorizable_iterations() * self.vector_size_in_bytes)
    }

    /// Gets the byte offset of where the non-vectorizable elements begin.
    pub fn get_remainder_ptr_offset(&self) -> u64 {
        if self.get_remainder_bytes() > 0 {
            self.valid_bytes.saturating_sub(self.vector_size_in_bytes)
        } else {
            0
        }
    }
}
