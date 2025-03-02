use smallvec::SmallVec;
extern crate alloc;
use alloc::vec::{IntoIter, Vec};
use core::cmp;
use core::fmt::{Debug, Formatter, Result as FmtResult};
use core::iter::FromIterator;
use core::ops::Range;
use core::slice::Iter;
use serde::{Deserialize, Serialize};

/// An element of an interval tree.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Element<K, V> {
    /// The range associated with this element.
    pub range: Range<K>,
    /// The value associated with this element.
    pub value: V,
}

impl<K, V> From<(Range<K>, V)> for Element<K, V> {
    fn from(tup: (Range<K>, V)) -> Element<K, V> {
        let (range, value) = tup;
        Element { range, value }
    }
}

#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
struct Node<K, V> {
    element: Element<K, V>,
    max: K,
}

/// A simple and generic implementation of an immutable interval tree.
///
/// To build it, always use `FromIterator`. This is not very optimized
/// as it takes `O(log n)` stack (it uses recursion) but runs in `O(n log n)`.
#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
pub struct IntervalTree<K, V> {
    data: Vec<Node<K, V>>,
}

impl<K: Ord + Clone, V, I: Into<Element<K, V>>> FromIterator<I> for IntervalTree<K, V> {
    fn from_iter<T: IntoIterator<Item = I>>(iter: T) -> Self {
        let mut nodes: Vec<_> = iter
            .into_iter()
            .map(|i| i.into())
            .map(|element| Node {
                max: element.range.end.clone(),
                element,
            })
            .collect();

        nodes.sort_unstable_by(|a, b| a.element.range.start.cmp(&b.element.range.start));

        if !nodes.is_empty() {
            Self::update_max(&mut nodes);
        }

        IntervalTree { data: nodes }
    }
}

/// An iterator over all the elements in the tree (in no particular order).
pub struct TreeIter<'a, K: 'a, V: 'a>(Iter<'a, Node<K, V>>);

impl<'a, K: 'a, V: 'a> Iterator for TreeIter<'a, K, V> {
    type Item = &'a Element<K, V>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|x| &x.element)
    }
}

impl<'a, K: 'a + Ord, V: 'a> IntoIterator for &'a IntervalTree<K, V> {
    type Item = &'a Element<K, V>;
    type IntoIter = TreeIter<'a, K, V>;

    fn into_iter(self) -> TreeIter<'a, K, V> {
        self.iter()
    }
}

/// An iterator that moves out of an interval tree.
pub struct TreeIntoIter<K, V>(IntoIter<Node<K, V>>);

impl<K, V> IntoIterator for IntervalTree<K, V> {
    type Item = Element<K, V>;
    type IntoIter = TreeIntoIter<K, V>;

    fn into_iter(self) -> TreeIntoIter<K, V> {
        TreeIntoIter(self.data.into_iter())
    }
}

impl<K, V> Iterator for TreeIntoIter<K, V> {
    type Item = Element<K, V>;

    fn next(&mut self) -> Option<Element<K, V>> {
        self.0.next().map(|x| x.element)
    }
}

impl<K: Ord + Clone, V> IntervalTree<K, V> {
    fn update_max(nodes: &mut [Node<K, V>]) -> K {
        assert!(!nodes.is_empty());
        let i = nodes.len() / 2;
        if nodes.len() > 1 {
            {
                let (left, rest) = nodes.split_at_mut(i);
                if !left.is_empty() {
                    rest[0].max = cmp::max(rest[0].max.clone(), Self::update_max(left));
                }
            }

            {
                let (rest, right) = nodes.split_at_mut(i + 1);
                if !right.is_empty() {
                    rest[i].max = cmp::max(rest[i].max.clone(), Self::update_max(right));
                }
            }
        }

        nodes[i].max.clone()
    }
}

impl<K: Ord, V> IntervalTree<K, V> {
    fn todo(&self) -> TodoVec {
        let mut todo = SmallVec::new();
        if !self.data.is_empty() {
            todo.push((0, self.data.len()));
        }
        todo
    }

    /// Queries the interval tree for all elements overlapping a given interval.
    ///
    /// This runs in `O(log n + m)`.
    pub fn query(
        &self,
        range: Range<K>,
    ) -> QueryIter<K, V> {
        QueryIter {
            todo: self.todo(),
            tree: self,
            query: Query::Range(range),
        }
    }

    /// Queries the interval tree for all elements containing a given point.
    ///
    /// This runs in `O(log n + m)`.
    pub fn query_point(
        &self,
        point: K,
    ) -> QueryIter<K, V> {
        QueryIter {
            todo: self.todo(),
            tree: self,
            query: Query::Point(point),
        }
    }

    /// Returns an iterator over all elements in the tree (in no particular order).
    pub fn iter(&self) -> TreeIter<K, V> {
        TreeIter(self.data.iter())
    }

    /// Returns an iterator over all elements in the tree, sorted by `Element.range.start`.
    ///
    /// This is currently identical to `IntervalTree::iter` because the internal structure
    /// is already sorted this way, but may not be in the future.
    pub fn iter_sorted(&self) -> impl Iterator<Item = &Element<K, V>> {
        TreeIter(self.data.iter())
    }
}

#[derive(Clone)]
enum Query<K> {
    Point(K),
    Range(Range<K>),
}

impl<K: Ord> Query<K> {
    fn point(&self) -> &K {
        match *self {
            Query::Point(ref key) => key,
            Query::Range(ref range) => &range.start,
        }
    }

    fn go_right(
        &self,
        start: &K,
    ) -> bool {
        match *self {
            Query::Point(ref key) => key >= start,
            Query::Range(ref range) => &range.end > start,
        }
    }

    fn intersect(
        &self,
        range: &Range<K>,
    ) -> bool {
        match *self {
            Query::Point(ref key) => key < &range.end,
            Query::Range(ref range) => range.end > range.start && range.start < range.end,
        }
    }
}

type TodoVec = SmallVec<[(usize, usize); 16]>;

/// Iterator for query results.
pub struct QueryIter<'a, K: 'a, V: 'a> {
    tree: &'a IntervalTree<K, V>,
    todo: TodoVec,
    query: Query<K>,
}

impl<'a, K: Ord + Clone, V> Clone for QueryIter<'a, K, V> {
    fn clone(&self) -> Self {
        QueryIter {
            tree: self.tree,
            todo: self.todo.clone(),
            query: self.query.clone(),
        }
    }
}

impl<'a, K: Ord + Clone + Debug, V: Debug> Debug for QueryIter<'a, K, V> {
    fn fmt(
        &self,
        fmt: &mut Formatter,
    ) -> FmtResult {
        let v: Vec<_> = (*self).clone().collect();
        write!(fmt, "{:?}", v)
    }
}

impl<'a, K: Ord, V> Iterator for QueryIter<'a, K, V> {
    type Item = &'a Element<K, V>;

    fn next(&mut self) -> Option<&'a Element<K, V>> {
        while let Some((start_index, length)) = self.todo.pop() {
            let middle_index = start_index + length / 2;
            let node = &self.tree.data[middle_index];

            if self.query.point() < &node.max {
                // Check the left subtree.
                {
                    let left_subtree_size = middle_index - start_index;
                    if left_subtree_size > 0 {
                        self.todo.push((start_index, left_subtree_size));
                    }
                }

                if self.query.go_right(&node.element.range.start) {
                    // Check the right subtree.
                    {
                        let right_subtree_size = length + start_index - middle_index - 1;
                        if right_subtree_size > 0 {
                            self.todo.push((middle_index + 1, right_subtree_size));
                        }
                    }

                    // Finally, check the current node.
                    if self.query.intersect(&node.element.range) {
                        return Some(&node.element);
                    }
                }
            }
        }

        None
    }
}
