#![allow(non_camel_case_types)]

use super::navigator::*;
use std::marker::PhantomData;
use std::mem;

const CURSOR_INVALID_POS: usize = ::std::usize::MAX;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct node(usize);

impl node {
    #[inline]
    pub(crate) fn new_invalid() -> Self {
        node(CURSOR_INVALID_POS)
    }

    #[inline]
    pub(crate) unsafe fn new_with_idx(idx: usize) -> Self {
        node(idx)
    }

    #[inline]
    pub(crate) fn is_invalid(self) -> bool {
        self.0 == CURSOR_INVALID_POS
    }

    #[inline]
    pub(crate) fn into_opt_node(self) -> Option<node> {
        if self.0 == CURSOR_INVALID_POS {
            None
        } else {
            Some(self)
        }
    }

    #[inline]
    pub(crate) fn into_opt_idx(self) -> Option<usize> {
        if self.0 == CURSOR_INVALID_POS {
            None
        } else {
            Some(self.0)
        }
    }
}

#[derive(Clone)]
pub(crate) struct ForestEntry<T> {
    pub(crate) data: T,
    pub(crate) parent: node,
    pub(crate) prev: node,
    pub(crate) next: node,
    pub(crate) child_first: node,
    pub(crate) child_last: node,
}

impl<T> ForestEntry<T> {
    pub(crate) fn new(v: T) -> Self {
        ForestEntry {
            data: v,
            parent: node::new_invalid(),
            prev: node::new_invalid(),
            next: node::new_invalid(),
            child_first: node::new_invalid(),
            child_last: node::new_invalid(),
        }
    }
}

pub struct forest<T> {
    data: [ForestEntry<T>],
}

impl<T> forest<T> {
    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn iter(&self) -> Iter<T> {
        Iter::new(self)
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut::new(self)
    }

    pub(crate) fn get_inner_ptr(&self, cursor: node) -> *const T {
        if let Some(idx) = cursor.into_opt_idx() {
            unsafe {
                let entry: &ForestEntry<T> = self.data.get_unchecked(idx);
                &entry.data
            }
        } else {
            ::std::ptr::null()
        }
    }

    pub(crate) fn get_inner_ptr_mut(&mut self, cursor: node) -> *mut T {
        if let Some(idx) = cursor.into_opt_idx() {
            unsafe {
                let entry: &mut ForestEntry<T> = self.data.get_unchecked_mut(idx);
                &mut entry.data
            }
        } else {
            ::std::ptr::null_mut()
        }
    }
}

#[repr(C)]
struct ForestRefRepr<T> {
    pub data: *const T,
    pub len: usize,
}

#[inline]
pub(crate) unsafe fn forest_ref_from_raw_parts<'a, T>(
    p: *const ForestEntry<T>,
    len: usize,
) -> &'a forest<T> {
    mem::transmute(ForestRefRepr { data: p, len: len })
}

#[inline]
pub(crate) unsafe fn forest_ref_from_raw_parts_mut<'a, T>(
    p: *mut ForestEntry<T>,
    len: usize,
) -> &'a mut forest<T> {
    mem::transmute(ForestRefRepr { data: p, len: len })
}

#[derive(Copy, Clone)]
pub enum IterMode {
    PreOrder,
    PostOrder,
    Both,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum IterMovement {
    None,
    GotoStart,
    GotoEnd,
    Left,
    Right,
    DownFirst(usize),
    DownLast(usize),
    Up(usize),
    UpNRight(usize),
    UpNLeft(usize),
    RightDownFirstN(usize),
    LeftDownLastN(usize),
}

impl IterMovement {
    fn join(self, other: Self) -> Self {
        match (self, other) {
            (IterMovement::None, b) => b,
            (b, IterMovement::None) => b,
            (_, IterMovement::GotoStart) => IterMovement::GotoStart,
            (_, IterMovement::GotoEnd) => IterMovement::GotoEnd,
            (IterMovement::Up(a), IterMovement::Up(b)) => IterMovement::Up(a + b),
            (IterMovement::DownFirst(a), IterMovement::DownFirst(b)) => {
                IterMovement::DownFirst(a + b)
            }
            (IterMovement::DownLast(a), IterMovement::DownLast(b)) => IterMovement::DownLast(a + b),
            (IterMovement::Up(a), IterMovement::Left) => IterMovement::UpNLeft(a),
            (IterMovement::Up(a), IterMovement::Right) => IterMovement::UpNRight(a),
            (IterMovement::Up(a), IterMovement::UpNLeft(b)) => IterMovement::UpNLeft(a + b),
            (IterMovement::Up(a), IterMovement::UpNRight(b)) => IterMovement::UpNRight(a + b),
            (IterMovement::Left, IterMovement::DownLast(a)) => IterMovement::LeftDownLastN(a),
            (IterMovement::Right, IterMovement::DownFirst(a)) => IterMovement::RightDownFirstN(a),
            (IterMovement::LeftDownLastN(a), IterMovement::DownLast(b)) => {
                IterMovement::LeftDownLastN(a + b)
            }
            (IterMovement::RightDownFirstN(a), IterMovement::DownFirst(b)) => {
                IterMovement::RightDownFirstN(a + b)
            }
            _ => panic!("can't merge iterator movement {:?} and {:?}", self, other),
        }
    }
}

pub struct Iter<'a, T: 'a> {
    data: &'a forest<T>,
    mode: IterMode,
    cursor: (node, bool),
}

pub struct IterMut<'a, T: 'a> {
    data: &'a mut forest<T>,
    mode: IterMode,
    cursor: (node, bool),
}

impl<'a, T: 'a> Iter<'a, T> {
    fn new(f: &'a forest<T>) -> Self {
        Iter {
            data: f,
            mode: IterMode::PreOrder,
            cursor: (node::new_invalid(), true),
        }
    }

    pub fn mode(mut self, mode: IterMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn values(self) -> Values<&'a T, Self> {
        Values {
            iter: self,
            phantom: PhantomData,
        }
    }

    pub fn last_visited_node(&self) -> Option<node> {
        if self.cursor.0.is_invalid() {
            None
        } else {
            Some(self.cursor.0)
        }
    }
}

impl<'a, T> IterMut<'a, T> {
    fn new(f: &'a mut forest<T>) -> Self {
        IterMut {
            data: f,
            mode: IterMode::PreOrder,
            cursor: (node::new_invalid(), true),
        }
    }

    pub fn mode(mut self, mode: IterMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn values(self) -> Values<&'a mut T, Self> {
        Values {
            iter: self,
            phantom: PhantomData,
        }
    }

    pub fn last_visited_node(&self) -> Option<node> {
        if self.cursor.0.is_invalid() {
            None
        } else {
            Some(self.cursor.0)
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum IterDir {
    Next,
    Prev,
}

impl<T> forest<T> {
    pub(crate) fn top_first_entry(&self) -> node {
        if self.len() == 0 {
            node::new_invalid()
        } else {
            let mut cur_idx = 0;

            loop {
                let cur_entry = &self.data[cur_idx];
                let parent_cursor = cur_entry.parent;
                if let Some(parent_idx) = parent_cursor.into_opt_idx() {
                    cur_idx = parent_idx;
                } else {
                    break;
                }
            }

            loop {
                let cur_entry = &self.data[cur_idx];
                let prev_cursor = cur_entry.prev;
                if let Some(prev_idx) = prev_cursor.into_opt_idx() {
                    cur_idx = prev_idx;
                } else {
                    break;
                }
            }

            unsafe { node::new_with_idx(cur_idx) }
        }
    }

    pub(crate) fn seek_entry(&self, pos: SeekPos) -> node {
        let mut cur = self.top_first_entry();
        if cur.is_invalid() {
            return cur;
        }

        match pos {
            SeekPos::TopLast | SeekPos::BottomLast => loop {
                let cur_entry = &self.data[cur.into_opt_idx().unwrap()];
                let next_cursor = cur_entry.next;
                if next_cursor.is_invalid() {
                    break;
                }
                cur = next_cursor;
            },
            SeekPos::TopFirst | SeekPos::BottomFirst => {}
        }

        match pos {
            SeekPos::TopFirst | SeekPos::TopLast => {}
            SeekPos::BottomFirst => loop {
                let cur_entry = &self.data[cur.into_opt_idx().unwrap()];
                let child_cursor = cur_entry.child_first;
                if child_cursor.is_invalid() {
                    break;
                }
                cur = child_cursor;
            },
            SeekPos::BottomLast => loop {
                let cur_entry = &self.data[cur.into_opt_idx().unwrap()];
                let child_cursor = cur_entry.child_last;
                if child_cursor.is_invalid() {
                    break;
                }
                cur = child_cursor;
            },
        }
        cur
    }

    pub(crate) fn iterate_once(
        &self,
        (cursor, entry): (node, bool),
        dir: IterDir,
    ) -> (IterMovement, (node, bool)) {
        if let Some(idx) = cursor.into_opt_idx() {
            let cur_entry = &self.data[idx];
            match (entry, dir) {
                (true, IterDir::Next) => {
                    let new_cursor = cur_entry.child_first;
                    if new_cursor.is_invalid() {
                        (IterMovement::None, (cursor, false))
                    } else {
                        (IterMovement::DownFirst(1), (new_cursor, true))
                    }
                }
                (false, IterDir::Next) => {
                    let mut new_cursor = cur_entry.next;
                    if new_cursor.is_invalid() {
                        new_cursor = cur_entry.parent;
                        if new_cursor.is_invalid() {
                            (IterMovement::GotoEnd, (new_cursor, false))
                        } else {
                            (IterMovement::Up(1), (new_cursor, false))
                        }
                    } else {
                        (IterMovement::Right, (new_cursor, true))
                    }
                }
                (true, IterDir::Prev) => {
                    let mut new_cursor = cur_entry.child_last;
                    if new_cursor.is_invalid() {
                        new_cursor = cur_entry.prev;
                        if new_cursor.is_invalid() {
                            (IterMovement::GotoStart, (new_cursor, true))
                        } else {
                            (IterMovement::Left, (new_cursor, false))
                        }
                    } else {
                        (IterMovement::DownLast(1), (new_cursor, false))
                    }
                }
                (false, IterDir::Prev) => {
                    let new_cursor = cur_entry.child_last;
                    if new_cursor.is_invalid() {
                        (IterMovement::None, (cursor, true))
                    } else {
                        (IterMovement::DownLast(1), (new_cursor, false))
                    }
                }
            }
        } else {
            match (entry, dir) {
                (true, IterDir::Next) => {
                    let new_cursor = self.seek_entry(SeekPos::TopFirst);
                    if new_cursor.is_invalid() {
                        (IterMovement::None, (cursor, entry))
                    } else {
                        (IterMovement::DownFirst(0), (new_cursor, true))
                    }
                }
                (false, IterDir::Prev) => {
                    let new_cursor = self.seek_entry(SeekPos::TopLast);
                    if new_cursor.is_invalid() {
                        (IterMovement::None, (cursor, entry))
                    } else {
                        (IterMovement::DownLast(0), (new_cursor, false))
                    }
                }
                _ => (IterMovement::None, (cursor, entry)),
            }
        }
    }

    pub(crate) fn iterate_entry(
        &self,
        (mut cur_cursor, mut cur_entry): (node, bool),
        dir: IterDir,
        mode: IterMode,
    ) -> (IterMovement, (node, bool)) {
        let mut movement = IterMovement::None;
        loop {
            let (new_movement, (new_cursor, new_entry)) =
                self.iterate_once((cur_cursor, cur_entry), dir);
            movement = movement.join(new_movement);
            cur_cursor = new_cursor;
            cur_entry = new_entry;
            match movement {
                | IterMovement::GotoStart | IterMovement::GotoEnd => {
                    return (movement, (cur_cursor, cur_entry));
                }
                _ => match (cur_entry, mode) {
                    | (true, IterMode::PostOrder) | (false, IterMode::PreOrder) => continue,
                    _ => {
                        return (movement, (cur_cursor, cur_entry));
                    }
                },
            }
        }
    }

    pub(crate) fn navigate_entry(&self, cursor: node, dir: NavigateDir) -> Option<(node, bool)> {
        let idx = cursor.into_opt_idx()?;
        let cur_entry = &self.data[idx];
        let (new_cursor_idx, new_entry) = match dir {
            NavigateDir::Up(up_entry) => (cur_entry.parent.into_opt_idx()?, up_entry),
            NavigateDir::Down => (cur_entry.child_first.into_opt_idx()?, true),
            NavigateDir::Left => (cur_entry.prev.into_opt_idx()?, true),
            NavigateDir::Right => (cur_entry.next.into_opt_idx()?, true),
        };
        Some((unsafe { node::new_with_idx(new_cursor_idx) }, new_entry))
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = (IterMovement, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        let (movement, new_pair) = self.data
            .iterate_entry(self.cursor, IterDir::Next, self.mode);
        self.cursor = new_pair;

        let valueref = unsafe { self.data.get_inner_ptr(new_pair.0).as_ref() }?;

        Some((movement, valueref))
    }
}

impl<'a, T> BiIterator for Iter<'a, T> {
    fn prev(&mut self) -> Option<Self::Item> {
        let (movement, new_pair) = self.data
            .iterate_entry(self.cursor, IterDir::Prev, self.mode);
        self.cursor = new_pair;

        let valueref = unsafe { self.data.get_inner_ptr(new_pair.0).as_ref() }?;

        Some((movement, valueref))
    }
}

impl<'a, T> Navigator for Iter<'a, T> {
    type Item = &'a T;

    fn navigate(&mut self, dir: NavigateDir) -> Option<Self::Item> {
        let new_pair = self.data.navigate_entry(self.cursor.0, dir)?;
        self.cursor = new_pair;

        let valueref = unsafe { self.data.get_inner_ptr(new_pair.0).as_ref() }?;
        Some(valueref)
    }

    fn seek(&mut self, pos: SeekPos) -> Option<Self::Item> {
        let new_pos = self.data.seek_entry(pos);
        if new_pos.is_invalid() {
            return None;
        };
        let new_entry = match self.mode {
            IterMode::PreOrder => true,
            IterMode::PostOrder => false,
            IterMode::Both => match pos {
                SeekPos::TopFirst | SeekPos::BottomFirst => true,
                SeekPos::TopLast | SeekPos::BottomLast => false,
            },
        };
        self.cursor = (new_pos, new_entry);

        let valueref = unsafe { self.data.get_inner_ptr(new_pos).as_ref() }?;
        Some(valueref)
    }
}

impl<'a, T: 'a> Iterator for IterMut<'a, T> {
    type Item = (IterMovement, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        let (movement, new_pair) = self.data
            .iterate_entry(self.cursor, IterDir::Next, self.mode);
        self.cursor = new_pair;

        let valueref = unsafe { self.data.get_inner_ptr_mut(new_pair.0).as_mut() }?;

        Some((movement, valueref))
    }
}

impl<'a, T: 'a> BiIterator for IterMut<'a, T> {
    fn prev(&mut self) -> Option<Self::Item> {
        let (movement, new_pair) = self.data
            .iterate_entry(self.cursor, IterDir::Prev, self.mode);
        self.cursor = new_pair;

        let valueref = unsafe { self.data.get_inner_ptr_mut(new_pair.0).as_mut() }?;

        Some((movement, valueref))
    }
}

impl<'a, T> Navigator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn navigate(&mut self, dir: NavigateDir) -> Option<Self::Item> {
        let new_pair = self.data.navigate_entry(self.cursor.0, dir)?;
        self.cursor = new_pair;

        let valueref = unsafe { self.data.get_inner_ptr_mut(new_pair.0).as_mut() }?;
        Some(valueref)
    }

    fn seek(&mut self, pos: SeekPos) -> Option<Self::Item> {
        let new_pos = self.data.seek_entry(pos);
        if new_pos.is_invalid() {
            return None;
        };
        let new_entry = match self.mode {
            IterMode::PreOrder => true,
            IterMode::PostOrder => false,
            IterMode::Both => match pos {
                SeekPos::TopFirst | SeekPos::BottomFirst => true,
                SeekPos::TopLast | SeekPos::BottomLast => false,
            },
        };
        self.cursor = (new_pos, new_entry);

        let valueref = unsafe { self.data.get_inner_ptr_mut(new_pos).as_mut() }?;
        Some(valueref)
    }
}

#[derive(Clone)]
pub struct Values<V, T> {
    iter: T,
    phantom: PhantomData<V>,
}

impl<V, T: Iterator<Item = (IterMovement, V)>> Iterator for Values<V, T> {
    type Item = V;
    fn next(&mut self) -> Option<V> {
        self.iter.next().map(|x| x.1)
    }
}

impl<V, T: BiIterator<Item = (IterMovement, V)>> BiIterator for Values<V, T> {
    fn prev(&mut self) -> Option<V> {
        self.iter.prev().map(|x| x.1)
    }
}

impl<V, T: Navigator<Item = V>> Navigator for Values<V, T> {
    type Item = V;

    fn navigate(&mut self, dir: NavigateDir) -> Option<Self::Item> {
        self.iter.navigate(dir)
    }

    fn seek(&mut self, pos: SeekPos) -> Option<Self::Item> {
        self.iter.seek(pos)
    }
}
