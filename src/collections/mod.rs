use super::navigator::*;
use super::primitive::*;
use std::ops::{Deref, DerefMut};

macro_rules! intrinsics_assume {
    ($x:expr) => {}; //    ($x: expr) => {::std::intrinsics::assume($x);};
}

#[derive(Clone, Default)]
pub struct Forest<T> {
    data: Vec<ForestEntry<T>>,
}

impl<T> Forest<T> {
    pub fn new() -> Self {
        Forest { data: Vec::new() }
    }
}

impl<T> AsRef<forest<T>> for Forest<T> {
    fn as_ref(&self) -> &forest<T> {
        self.deref()
    }
}

impl<T> AsMut<forest<T>> for Forest<T> {
    fn as_mut(&mut self) -> &mut forest<T> {
        self.deref_mut()
    }
}

impl<T> Forest<T> {
    pub fn get_first_root_node(&self) -> Option<node> {
        self.seek_entry(SeekPos::TopFirst).into_opt_node()
    }

    fn check_node_validity(&self, n: node) -> usize {
        let len = self.len();
        n.into_opt_idx()
            .and_then(|x| if x < len { Some(x) } else { None })
            .expect("node out of bound")
    }

    pub fn get_parent_node(&self, n: node) -> Option<node> {
        let cur_idx = self.check_node_validity(n);
        let cur_entry = self.data.get(cur_idx).unwrap();
        cur_entry.parent.into_opt_node()
    }

    pub fn get_prev_sibling_node(&self, n: node) -> Option<node> {
        let cur_idx = self.check_node_validity(n);
        let cur_entry = self.data.get(cur_idx).unwrap();
        cur_entry.prev.into_opt_node()
    }

    pub fn get_next_sibling_node(&self, n: node) -> Option<node> {
        let cur_idx = self.check_node_validity(n);
        let cur_entry = self.data.get(cur_idx).unwrap();
        cur_entry.next.into_opt_node()
    }

    pub fn get_first_child_node(&self, n: node) -> Option<node> {
        let cur_idx = self.check_node_validity(n);
        let cur_entry = self.data.get(cur_idx).unwrap();
        cur_entry.child_first.into_opt_node()
    }

    pub fn get_last_child_node(&self, n: node) -> Option<node> {
        let cur_idx = self.check_node_validity(n);
        let cur_entry = self.data.get(cur_idx).unwrap();
        cur_entry.child_last.into_opt_node()
    }
}

impl<T> Forest<T> {
    fn check_node_not_equal_or_ancestor_of_another(&self, a: node, b: node) -> bool {
        let target_idx = self.check_node_validity(a);
        let mut cur_idx = self.check_node_validity(b);
        if cur_idx == target_idx {
            return false;
        }
        loop {
            let cur_entry = self.data.get(cur_idx).unwrap();
            if let Some(parent_idx) = cur_entry.parent.into_opt_idx() {
                if parent_idx == target_idx {
                    return false;
                } else {
                    cur_idx = parent_idx;
                    continue;
                }
            } else {
                return true;
            }
        }
    }

    fn reconnect_prev_next(&mut self, new_prev: node, cur: node, new_next: node) {
        {
            let cur_idx = cur.into_opt_idx().unwrap();
            let cur_entry = self.data.get_mut(cur_idx).unwrap();
            cur_entry.prev = new_prev;
            cur_entry.next = new_next;
        }
        if let Some(prev_idx) = new_prev.into_opt_idx() {
            let prev_entry = self.data.get_mut(prev_idx).unwrap();
            prev_entry.next = cur;
        }
        if let Some(next_idx) = new_next.into_opt_idx() {
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

    fn disconnect_node_from_parent(&mut self, cur: node, parent: node) {
        let cur_idx = cur.into_opt_idx().unwrap();
        let parent_idx = parent.into_opt_idx().unwrap();

        let (cur_prev, cur_next) = {
            let cur_entry = self.data.get_mut(cur_idx).unwrap();
            cur_entry.parent = node::new_invalid();
            (cur_entry.prev, cur_entry.next)
        };

        let parent_entry = self.data.get_mut(parent_idx).unwrap();
        if parent_entry.child_first == cur {
            parent_entry.child_first = cur_next;
        }
        if parent_entry.child_last == cur {
            parent_entry.child_last = cur_prev;
        }
    }

    fn knockout_node_from_siblings(&mut self, cur: node) {
        let cur_idx = cur.into_opt_idx().unwrap();
        let (cur_prev, cur_next) = {
            let cur_entry = self.data.get_mut(cur_idx).unwrap();
            let pair = (cur_entry.prev, cur_entry.next);
            cur_entry.prev = node::new_invalid();
            cur_entry.next = node::new_invalid();
            pair
        };
        if let Some(cur_prev_idx) = cur_prev.into_opt_idx() {
            let prev_entry = self.data.get_mut(cur_prev_idx).unwrap();
            prev_entry.next = cur_next;
        }
        if let Some(cur_next_idx) = cur_next.into_opt_idx() {
            let next_entry = self.data.get_mut(cur_next_idx).unwrap();
            next_entry.prev = cur_prev;
        }
    }

    fn move_detached_node(&mut self, cur: node, new_parent: node, new_prev: node, new_next: node) {
        let cur_idx = cur.into_opt_idx().unwrap();
        {
            let cur_entry = self.data.get_mut(cur_idx).unwrap();
            cur_entry.parent = new_parent;
        }
        let parent_idx = new_parent.into_opt_idx().unwrap();
        {
            let parent_entry = self.data.get_mut(parent_idx).unwrap();
            if parent_entry.child_first == new_next {
                parent_entry.child_first = cur;
            }
            if parent_entry.child_last == new_prev {
                parent_entry.child_last = cur;
            }
        }
        self.reconnect_prev_next(new_prev, cur, new_next);
    }
}

impl<T> Forest<T> {
    pub fn create_node(&mut self, t: T) -> node {
        self.prepare_new_node_at_top_last(t)
    }

    pub fn detach_node(&mut self, n: node) -> bool {
        // check_node_validity will execute inside get_parent_node()
        if let Some(parent_node) = self.get_parent_node(n) {
            // checkpoint, n is not toplevel
            let top_last_node = self.seek_entry(SeekPos::TopLast);

            self.disconnect_node_from_parent(n, parent_node);
            self.knockout_node_from_siblings(n);
            self.reconnect_prev_next(top_last_node, n, node::new_invalid());

            true
        } else {
            false
        }
    }

    pub fn prepend_node_child(&mut self, n: node, child: node) -> bool {
        if self.check_node_not_equal_or_ancestor_of_another(child, n) {
            let _ = self.detach_node(child);
            let new_prev = node::new_invalid();
            let new_next = self.get_first_child_node(n)
                .unwrap_or_else(node::new_invalid);
            self.knockout_node_from_siblings(child);
            self.move_detached_node(child, n, new_prev, new_next);
            true
        } else {
            false
        }
    }

    pub fn append_node_child(&mut self, n: node, child: node) -> bool {
        if self.check_node_not_equal_or_ancestor_of_another(child, n) {
            let _ = self.detach_node(child);
            let new_next = node::new_invalid();
            let new_prev = self.get_last_child_node(n)
                .unwrap_or_else(node::new_invalid);
            self.knockout_node_from_siblings(child);
            self.move_detached_node(child, n, new_prev, new_next);
            true
        } else {
            false
        }
    }

    pub fn insert_node_child_before(&mut self, n: node, child: node, referent: node) -> bool {
        if self.check_node_not_equal_or_ancestor_of_another(child, n)
            && self.get_parent_node(referent) == Some(n) && child != referent
        {
            let _ = self.detach_node(child);
            let new_next = referent;
            let new_prev = self.get_prev_sibling_node(referent)
                .unwrap_or_else(node::new_invalid);
            self.knockout_node_from_siblings(child);
            self.move_detached_node(child, n, new_prev, new_next);
            true
        } else {
            false
        }
    }

    pub fn insert_node_child_after(&mut self, n: node, child: node, referent: node) -> bool {
        if self.check_node_not_equal_or_ancestor_of_another(child, n)
            && self.get_parent_node(referent) == Some(n) && child != referent
        {
            let _ = self.detach_node(child);
            let new_prev = referent;
            let new_next = self.get_next_sibling_node(referent)
                .unwrap_or_else(node::new_invalid);
            self.knockout_node_from_siblings(child);
            self.move_detached_node(child, n, new_prev, new_next);
            true
        } else {
            false
        }
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
