use super::super::primitive::*;
use super::super::navigator::*;
use std::ops::{Deref, DerefMut};

macro_rules! intrinsics_assume {
    ($x: expr) => {};
//    ($x: expr) => {::std::intrinsics::assume($x);};
}

pub struct Forest<T> {
    data: Vec<ForestEntry<T>>,
}

impl<T> Forest<T> {
    pub fn new() -> Self {
        Forest {
            data: Vec::new(),
        }
    }

    pub fn as_ref(&self) -> &forest<T> {
        self.deref()
    }

    pub fn as_mut(&mut self) -> &mut forest<T> {
        self.deref_mut()
    }
}



impl<T> Forest<T> {
    fn reconnect_prev_next(&mut self, new_prev: node, cur: node, new_next: node) {
        {
            let cur_idx = cur.into_opt().unwrap();
            let cur_entry = self.data.get_mut(cur_idx).unwrap();
            cur_entry.prev = new_prev;
            cur_entry.next = new_next;
        }
        if let Some(prev_idx) = new_prev.into_opt() {
            let prev_entry = self.data.get_mut(prev_idx).unwrap();
            prev_entry.next = cur;
        }
        if let Some(next_idx) = new_next.into_opt() {
            let next_entry = self.data.get_mut(next_idx).unwrap();
            next_entry.prev = cur;
        }
    }

    fn prepare_new_node_at_top_last(&mut self, t: T) -> node {
        let top_last_node = self.seek_entry(SeekPos::TopLast);
        let new_entry = ForestEntry::new(t);
        let new_idx = self.data.len();
        self.data.push(new_entry);
        let new_node = unsafe { node::new_with_idx(new_idx) };
        self.reconnect_prev_next(top_last_node, new_node, node::new_invalid());
        new_node
    }
}



impl<T> Forest<T> {
    pub fn create_node(&mut self, t: T) -> node {
        self.prepare_new_node_at_top_last(t)
    }
}

impl<T> Deref for Forest<T> {
    type Target = forest<T>;

    fn deref(&self) -> &forest<T> {
        unsafe {
            let p = self.data.as_ptr();
            intrinsics_assume!(!p.is_null());
            forest_ref_from_raw_parts(p, self.data.len())
        }
    }
}

impl<T> DerefMut for Forest<T> {
    fn deref_mut(&mut self) -> &mut forest<T> {
        unsafe {
            let p = self.data.as_mut_ptr();
            intrinsics_assume!(!p.is_null());
            forest_ref_from_raw_parts_mut(p, self.data.len())
        }
    }
}

