use std::iter::FromIterator;
use result::Result::{self, None, Value, Start, Finish};

pub trait ForestIterator {
    type Item;

    fn next(&mut self) -> Result<Self::Item>;

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Option::None)
    }

    fn next_start(&mut self) -> Result<Self::Item> {
        loop {
            match self.next() {
                n @ None | n @ Start { .. } => return n,
                _ => continue,
            };
        }
    }

    fn next_value(&mut self) -> Result<Self::Item> {
        loop {
            match self.next() {
                n @ None | n @ Value { .. } => return n,
                _ => continue,
            };
        }
    }

    fn goto_value_or_finish(&mut self) -> Result<Self::Item> {
        let mut nest_counter = 0usize;
        loop {
            match self.next() {
                n @ None => return n,
                Start { .. } => nest_counter += 1,
                n @ Finish { .. } => {
                    if nest_counter == 0 {
                        return n;
                    } else {
                        nest_counter -= 1
                    }
                }
                n @ Value { .. } => {
                    if nest_counter == 0 {
                        return n;
                    }
                }
            }
        }
    }

    fn goto_finish(&mut self) -> Result<Self::Item> {
        let mut nest_counter = 0usize;
        loop {
            match self.next() {
                n @ None => return n,
                Start { .. } => nest_counter += 1,
                n @ Finish { .. } => {
                    if nest_counter == 0 {
                        return n;
                    } else {
                        nest_counter -= 1
                    }
                }
                Value { .. } => {}
            }
        }
    }

    fn count_values(self) -> usize
    where
        Self: Sized,
    {
        let mut counter = 0usize;
        let mut iter = self;
        loop {
            match iter.next() {
                None => return counter,
                Value { .. } => counter += 1,
                _ => {}
            }
        }
    }

    fn collect<B>(self) -> B
    where
        Self: Sized,
        B: FromForestIterator<Self::Item>,
    {
        B::from_iter(self)
    }

    fn collect_values<B>(self) -> B
    where
        Self: Sized,
        B: FromIterator<Self::Item>,
    {
        B::from_iter(self.values())
    }

    fn peekable(self) -> Peekable<Self>
    where
        Self: Sized,
    {
        Peekable {
            iter: self,
            peeked: Option::None,
        }
    }

    fn values(self) -> Values<Self>
    where
        Self: Sized,
    {
        Values { iter: self }
    }
}

pub struct Peekable<I: ForestIterator> {
    iter: I,
    peeked: Option<Result<I::Item>>,
}

impl<I: ForestIterator> Peekable<I> {
    pub fn peek(&mut self) -> Result<&<I as ForestIterator>::Item> {
        if self.peeked.is_none() {
            self.peeked = Some(self.iter.next());
        };
        self.peeked.as_ref().unwrap().as_ref()
    }
}

impl<I: ForestIterator> ForestIterator for Peekable<I> {
    type Item = I::Item;

    fn next(&mut self) -> Result<Self::Item> {
        if let Some(p) = self.peeked.take() {
            return p;
        }
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

pub struct Values<I: ForestIterator> {
    iter: I,
}

impl<I: ForestIterator> Iterator for Values<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next_value().value()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

pub trait IntoForestIterator {
    type Item;
    type IntoForestIter: ForestIterator<Item = Self::Item>;

    fn into_iter(self) -> Self::IntoForestIter;
}

impl<I> IntoForestIterator for I
where
    I: ForestIterator,
{
    type Item = <I as ForestIterator>::Item;
    type IntoForestIter = I;

    fn into_iter(self) -> Self::IntoForestIter {
        self
    }
}

pub trait FromForestIterator<A>: Sized {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoForestIterator<Item = A>;
}
