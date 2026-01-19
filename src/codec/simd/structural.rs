//! Structural index for SIMD parsing.
//!
//! The structural index contains positions of all structural characters
//! in the input, allowing the parser to jump directly to relevant positions
//! rather than scanning byte-by-byte.

/// Index of structural character positions in the input.
///
/// Structural characters in Bencodex include:
/// - Type markers: `n`, `t`, `f`, `i`, `l`, `d`, `u`
/// - Delimiters: `:`, `e`
/// - Digits: `0-9`
#[derive(Debug, Clone)]
pub struct StructuralIndex {
    /// Sorted list of positions where structural characters appear
    pub indices: Vec<u32>,
}

impl StructuralIndex {
    /// Create a new empty structural index
    pub fn new() -> Self {
        Self {
            indices: Vec::new(),
        }
    }

    /// Create a structural index with pre-allocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            indices: Vec::with_capacity(capacity),
        }
    }

    /// Get the number of structural characters found
    #[inline]
    pub fn len(&self) -> usize {
        self.indices.len()
    }

    /// Check if the index is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    /// Get position at given index
    #[inline]
    pub fn get(&self, idx: usize) -> Option<u32> {
        self.indices.get(idx).copied()
    }

    /// Add a position to the index
    #[inline]
    pub fn push(&mut self, pos: u32) {
        self.indices.push(pos);
    }

    /// Clear the index for reuse
    #[inline]
    pub fn clear(&mut self) {
        self.indices.clear();
    }
}

impl Default for StructuralIndex {
    fn default() -> Self {
        Self::new()
    }
}
