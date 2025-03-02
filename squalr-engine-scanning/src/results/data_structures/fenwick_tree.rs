/// Fenwick Tree (Binary Index Tree) for fast index queries.
pub struct FenwickTree {
    tree: Vec<u64>,
}

impl FenwickTree {
    /// Initializes a static Fenwick Tree from a presorted array.
    pub fn new(presorted_values: &[u64]) -> Self {
        let mut fenwick_tree = Self {
            tree: vec![0; presorted_values.len() + 1],
        };

        // First, initialize the fenwick_tree as a direct copy in O(n).
        for index in 1..fenwick_tree.tree.len() {
            fenwick_tree.tree[index] = presorted_values[index - 1];
        }

        fenwick_tree
    }

    /// Finds the smallest index `index` such that the sum of elements up to `index` is at least `k`.
    pub fn find_kth(
        &self,
        k: u64,
    ) -> Option<usize> {
        let mut index = 0;
        let mut sum = 0;
        let mut mask = self.tree.len().next_power_of_two() >> 1;

        while mask > 0 {
            let next_index = index + mask;
            if next_index < self.tree.len() && sum + self.tree[next_index] < k {
                sum += self.tree[next_index];
                index = next_index;
            }

            mask >>= 1;
        }

        if sum < k {
            None
        } else {
            // Return Some(index + 1) because Fenwick / Binary Index Trees are 1-based.
            Some(index + 1)
        }
    }
}
