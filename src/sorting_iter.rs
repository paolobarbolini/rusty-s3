use std::cmp::Ord;
use std::iter::{Fuse, FusedIterator};

#[derive(Debug, Clone)]
pub struct SortingIterator<A, B>
where
    A: Iterator,
    B: Iterator<Item = A::Item>,
{
    a: Fuse<A>,
    b: Fuse<B>,

    a_buffer: Option<A::Item>,
    b_buffer: Option<B::Item>,
}

impl<A, B> SortingIterator<A, B>
where
    A: Iterator,
    B: Iterator<Item = A::Item>,
{
    pub(crate) fn new(a: A, b: B) -> Self {
        Self {
            a: a.fuse(),
            b: b.fuse(),

            a_buffer: None,
            b_buffer: None,
        }
    }
}

impl<A, B> Iterator for SortingIterator<A, B>
where
    A: Iterator,
    B: Iterator<Item = A::Item>,
    A::Item: Ord,
    B::Item: Ord,
{
    type Item = A::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let a_next = self.a_buffer.take().or_else(|| self.a.next());
        let b_next = self.b_buffer.take().or_else(|| self.b.next());

        match (a_next, b_next) {
            (Some(a_next), Some(b_next)) if a_next < b_next => {
                self.b_buffer = Some(b_next);
                Some(a_next)
            }
            (Some(a_next), Some(b_next)) => {
                self.a_buffer = Some(a_next);
                Some(b_next)
            }
            (Some(next), None) | (None, Some(next)) => Some(next),
            (None, None) => None,
        }
    }
}

impl<A, B> FusedIterator for SortingIterator<A, B>
where
    A: Iterator,
    B: Iterator<Item = A::Item>,
    A::Item: Ord,
    B::Item: Ord,
{
}

#[cfg(test)]
mod tests {
    use std::iter;

    use super::*;

    #[test]
    fn empty() {
        let mut iter = SortingIterator::new(iter::empty::<u32>(), iter::empty::<u32>());
        assert!(iter.next().is_none());
    }

    #[test]
    fn numbers() {
        let a = vec![10, 20, 25, 30, 40];
        let b = vec![5, 9, 15, 35, 45, 50];

        let iter = SortingIterator::new(a.into_iter(), b.into_iter());
        assert!(iter.eq(vec![5, 9, 10, 15, 20, 25, 30, 35, 40, 45, 50].into_iter()))
    }

    #[test]
    fn numbers_empty() {
        let a = vec![10, 20, 25, 30, 40];

        let iter = SortingIterator::new(a.into_iter(), iter::empty());
        assert!(iter.eq(vec![10, 20, 25, 30, 40].into_iter()))
    }
}
