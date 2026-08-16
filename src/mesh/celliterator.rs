//! # Iterate over each cell of a mesh

/// A source of per-cell item data, stored as a single flat, contiguous
/// buffer that is implicitly divided into fixed-size groups ("cells").
///
/// # Memory layout contract
///
/// [`CellIterator`] has no concept of "cells" on its own - it only knows
/// how to carve `0..data_len()` into chunks of `point_per_cell()` items
/// at a time, and asks `point_data` for the slice at each step.
///
/// That means **the implementor is responsible for guaranteeing that all
/// items belonging to cell `c` are stored contiguously**, at the range
///
/// ```text
/// [c * point_per_cell() .. min((c + 1) * point_per_cell(), data_len())]
/// ```
///
/// i.e. cell 0's items come first, immediately followed by cell 1's
/// items, then cell 2's, and so on - a flattened "structure of arrays"
/// layout, with no gaps and no interleaving. If your storage doesn't
/// already look like this (e.g. cells are scattered, or items for one
/// cell are interspersed with another's), you must reorder the data
/// before exposing it through this trait - `CellIterator` cannot detect
/// or correct for a mismatched layout, it will simply yield slices that
/// don't correspond to the cells you intended.
///
/// Only the *last* cell is allowed to be shorter than
/// `point_per_cell()` - its length is `data_len() % point_per_cell()` (or
/// a full `point_per_cell()` when `data_len()` divides evenly). Every other
/// cell is always exactly `point_per_cell()` items.
///
/// # Other invariants
///
/// - `point_per_cell()` must be non-zero, and must return the same value
///   on every call for a given `self` (the iterator queries it
///   repeatedly rather than caching it).
/// - `data_len()` must match the number of items actually reachable via
///   `point_data`, and must likewise be stable across calls.
/// - `point_data(range)` must return exactly `range.len()` items, where
///   `point_data(a..b)[i]` corresponds to logical index `a + i`. Callers
///   (in particular [`CellIterator`]) only ever pass ranges that are
///   sub-ranges of `0..data_len()` and aligned to the cell boundaries
///   above - implementors don't need to validate or special-case
///   misaligned ranges, since none will arrive.
pub trait CellData {
    /// The type of a single item (e.g. a point, a normal, a color).
    type Item;

    /// How many items belong to one cell.
    ///
    /// It must be non-zero and constant for the lifetime of `self`.
    fn point_per_cell(&self) -> usize;

    /// Total number of items across *all* cells - i.e. the length of the
    /// flattened backing storage, not the number of cells.
    fn data_len(&self) -> usize;

    /// Slice of items for a given (already-clamped) range.
    ///
    /// `range` is always contained in `0..data_len()`, and (per the
    /// trait's layout contract) aligned to cell boundaries - so a
    /// correct implementation is simply slicing the contiguous backing
    /// storage by `range`.
    fn point_data(&self, range: std::ops::Range<usize>) -> &[Self::Item];
}

/// Iterates over a [`CellData`] source one cell at a time, yielding a
/// `&[P::Item]` slice per cell.
///
/// # What this assumes about `parent`
///
/// This iterator does no reordering or grouping of its own: it walks
/// `0..parent.data_len()` in fixed strides of `parent.point_per_cell()`
/// and fetches each stride via [`CellData::point_data`]. The resulting
/// slices only correspond to real cells if `parent`'s backing storage is
/// laid out **contiguously per cell**, as described in the
/// [`CellData`] layout contract. Violating that contract won't cause a
/// panic - it will just make this iterator silently yield slices that
/// mix items from different cells.
///
/// # Yielded chunks
///
/// Every chunk has exactly `point_per_cell()` items, except possibly the
/// final one, which is shorter when `data_len()` isn't an exact multiple
/// of `point_per_cell()`.
pub struct CellIterator<'a, P: CellData> {
    /// Provider of the data
    /// data are organized continuous in memory, with [CellData::point_per_cell] data per cell
    parent: &'a P,

    /// Current offset. Typically a multiple of [CellData::point_per_cell]
    offset: usize,
}

impl<'a, P: CellData> CellIterator<'a, P> {
    /// Creates an iterator starting at the first cell of `parent`.
    ///
    /// See the [`CellData`] layout contract for what `parent` must
    /// guarantee for the yielded slices to be meaningful.
    pub fn new(parent: &'a P) -> Self {
        Self { parent, offset: 0 }
    }
}

impl<'a, P: CellData> Iterator for CellIterator<'a, P> {
    type Item = &'a [P::Item];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let len = self.parent.data_len();
        if self.offset >= len {
            None
        } else {
            let per_cell = self.parent.point_per_cell();
            let end_offset = std::cmp::min(self.offset + per_cell, len);
            let chunk = self.parent.point_data(self.offset..end_offset);
            self.offset = end_offset;
            Some(chunk)
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let per_cell = self.parent.point_per_cell();
        assert_ne!(per_cell, 0);
        let remaining = self.parent.data_len().saturating_sub(self.offset);
        let n = remaining.div_ceil(per_cell);
        (n, Some(n))
    }

    #[inline]
    fn count(self) -> usize {
        let per_cell = self.parent.point_per_cell();
        assert_ne!(per_cell, 0);
        let len = self.parent.data_len();
        if self.offset >= len {
            0
        } else {
            (len - self.offset).div_ceil(per_cell)
        }
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        let per_cell = self.parent.point_per_cell();
        let len = self.parent.data_len();
        if per_cell == 0 || self.offset >= len {
            None
        } else {
            let last_offset = ((len - 1) / per_cell) * per_cell;
            Some(self.parent.point_data(last_offset..len))
        }
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let per_cell = self.parent.point_per_cell();
        self.offset = self.offset.saturating_add(n.saturating_mul(per_cell));
        self.next()
    }
}

impl<'a, P: CellData> ExactSizeIterator for CellIterator<'a, P> {}
impl<'a, P: CellData> std::iter::FusedIterator for CellIterator<'a, P> {}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal `CellData` fixture: a flat buffer of `u32`s grouped into
    /// fixed-size cells, used to drive `CellIterator` in isolation from
    /// any "real" point type.
    struct FlatData {
        items: Vec<u32>,
        per_cell: usize,
    }

    impl FlatData {
        fn new(items: Vec<u32>, per_cell: usize) -> Self {
            Self { items, per_cell }
        }
    }

    impl CellData for FlatData {
        type Item = u32;

        fn point_per_cell(&self) -> usize {
            self.per_cell
        }

        fn data_len(&self) -> usize {
            self.items.len()
        }

        fn point_data(&self, range: std::ops::Range<usize>) -> &[u32] {
            &self.items[range]
        }
    }

    #[test]
    fn empty_data_yields_no_cells() {
        let data = FlatData::new(vec![], 4);
        let mut iter = CellIterator::new(&data);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn exact_multiple_yields_uniform_chunks() {
        let data = FlatData::new(vec![10, 20, 30, 40, 50, 60], 2);
        let chunks: Vec<&[u32]> = CellIterator::new(&data).collect();
        assert_eq!(chunks, vec![&[10, 20][..], &[30, 40][..], &[50, 60][..]]);
    }

    #[test]
    fn remainder_yields_short_final_chunk() {
        let data = FlatData::new(vec![1, 2, 3, 4, 5, 6, 7], 3);
        let chunks: Vec<&[u32]> = CellIterator::new(&data).collect();
        assert_eq!(chunks, vec![&[1, 2, 3][..], &[4, 5, 6][..], &[7][..]]);
    }

    #[test]
    fn per_cell_larger_than_data_yields_single_short_chunk() {
        let data = FlatData::new(vec![1, 2], 5);
        let chunks: Vec<&[u32]> = CellIterator::new(&data).collect();
        assert_eq!(chunks, vec![&[1, 2][..]]);
    }

    #[test]
    fn size_hint_matches_remaining_cells() {
        let data = FlatData::new(vec![1, 2, 3, 4, 5, 6], 2);
        let mut iter = CellIterator::new(&data);
        assert_eq!(iter.size_hint(), (3, Some(3)));
        iter.next();
        assert_eq!(iter.size_hint(), (2, Some(2)));
        iter.next();
        iter.next();
        assert_eq!(iter.size_hint(), (0, Some(0)));
        // Querying past exhaustion should not underflow/panic.
        assert_eq!(iter.size_hint(), (0, Some(0)));
    }

    #[test]
    fn count_matches_number_of_chunks() {
        let data = FlatData::new(vec![1, 2, 3, 4, 5, 6, 7], 3);
        assert_eq!(CellIterator::new(&data).count(), 3);

        // count() after partial consumption only counts what's left.
        let mut iter = CellIterator::new(&data);
        iter.next();
        assert_eq!(iter.count(), 2);
    }

    #[test]
    fn last_returns_final_chunk_even_with_remainder() {
        let data = FlatData::new(vec![1, 2, 3, 4, 5, 6, 7], 3);
        let iter = CellIterator::new(&data);
        assert_eq!(iter.last(), Some(&[7][..]));
    }

    #[test]
    fn last_returns_final_chunk_on_exact_multiple() {
        let data = FlatData::new(vec![1, 2, 3, 4, 5, 6], 2);
        let iter = CellIterator::new(&data);
        assert_eq!(iter.last(), Some(&[5, 6][..]));
    }

    #[test]
    fn last_after_partial_consumption_is_still_correct() {
        let data = FlatData::new(vec![1, 2, 3, 4, 5, 6, 7, 8], 2);
        let mut iter = CellIterator::new(&data);
        iter.next(); // consume first chunk
        assert_eq!(iter.last(), Some(&[7, 8][..]));
    }

    #[test]
    fn last_on_empty_data_is_none() {
        let data = FlatData::new(vec![], 3);
        let iter = CellIterator::new(&data);
        assert_eq!(iter.last(), None);
    }

    #[test]
    fn nth_skips_whole_cells() {
        // 5 chunks of 2: [0,1] [2,3] [4,5] [6,7] [8,9]
        let data = FlatData::new((0..10).collect(), 2);
        let mut iter = CellIterator::new(&data);
        assert_eq!(iter.nth(2), Some(&[4, 5][..]));
        // Iterator should resume correctly from there.
        assert_eq!(iter.next(), Some(&[6, 7][..]));
    }

    #[test]
    fn nth_past_the_end_returns_none() {
        let data = FlatData::new(vec![1, 2, 3, 4], 2);
        let mut iter = CellIterator::new(&data);
        assert_eq!(iter.nth(10), None);
    }

    #[test]
    fn nth_with_huge_n_does_not_panic() {
        // Guards the saturating_mul/saturating_add overflow handling.
        let data = FlatData::new(vec![1, 2, 3, 4], 2);
        let mut iter = CellIterator::new(&data);
        assert_eq!(iter.nth(usize::MAX), None);
    }

    #[test]
    fn exact_size_iterator_len_matches_remaining_chunks() {
        let data = FlatData::new(vec![1, 2, 3, 4, 5, 6, 7], 3);
        let mut iter = CellIterator::new(&data);
        assert_eq!(iter.len(), 3);
        iter.next();
        assert_eq!(iter.len(), 2);
    }

    #[test]
    fn iterator_stays_exhausted_after_completion() {
        let data = FlatData::new(vec![1, 2, 3], 2);
        let mut iter = CellIterator::new(&data);
        assert!(iter.next().is_some());
        assert!(iter.next().is_some());
        assert_eq!(iter.next(), None);
        // Calling next() again after exhaustion keeps returning None.
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    #[should_panic]
    fn zero_point_per_cell_panics_on_size_hint() {
        let data = FlatData::new(vec![1, 2, 3], 0);
        let iter = CellIterator::new(&data);
        let _ = iter.size_hint();
    }

    /// A second `Item` type, to confirm `CellData`/`CellIterator` aren't
    /// accidentally coupled to `u32` or to `Copy` types.
    #[derive(Debug, Clone, PartialEq)]
    struct Pair {
        x: f32,
        y: f32,
    }

    struct FlatPairs {
        items: Vec<Pair>,
        per_cell: usize,
    }

    impl CellData for FlatPairs {
        type Item = Pair;

        fn point_per_cell(&self) -> usize {
            self.per_cell
        }

        fn data_len(&self) -> usize {
            self.items.len()
        }

        fn point_data(&self, range: std::ops::Range<usize>) -> &[Pair] {
            &self.items[range]
        }
    }

    #[test]
    fn works_with_non_primitive_item_type() {
        let items = vec![
            Pair { x: 0.0, y: 0.0 },
            Pair { x: 1.0, y: 1.0 },
            Pair { x: 2.0, y: 2.0 },
        ];
        let data = FlatPairs { items, per_cell: 2 };
        let chunks: Vec<&[Pair]> = CellIterator::new(&data).collect();
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].len(), 2);
        assert_eq!(chunks[1].len(), 1);
        assert_eq!(chunks[1][0], Pair { x: 2.0, y: 2.0 });
    }
}
