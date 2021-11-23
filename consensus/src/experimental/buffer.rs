// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::experimental::hashable::Hashable;
use diem_crypto::HashValue;
use std::collections::HashMap;

pub struct LinkedItem<T: Hashable> {
    // use option so we don't need T to be cloneable
    elem: Option<T>,
    // index is for find_element_by_key to have a starting position (similar to find_element)
    index: u64,
    next: Option<HashValue>,
}

pub type Cursor = Option<HashValue>;

/// Buffer implementes an ordered dictionary
/// It supports push_back, pop_front, and lookup by HashValue
pub struct Buffer<T: Hashable> {
    map: HashMap<HashValue, LinkedItem<T>>,
    count: u64,
    head: Cursor,
    tail: Cursor,
}

impl<T: Hashable> Buffer<T> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            count: 0,
            head: None,
            tail: None,
        }
    }

    pub fn head_cursor(&self) -> &Cursor {
        &self.head
    }

    #[cfg(test)]
    pub fn tail_cursor(&self) -> &Cursor {
        &self.tail
    }

    pub fn push_back(&mut self, elem: T) {
        self.count = self.count.checked_add(1).unwrap();
        let t_hash = elem.hash();
        self.map.insert(
            t_hash,
            LinkedItem {
                elem: Some(elem),
                index: self.count,
                next: None,
            },
        );
        if let Some(tail) = self.tail {
            self.map.get_mut(&tail).unwrap().next = Some(t_hash);
        }
        self.tail = Some(t_hash);
        self.head.get_or_insert(t_hash);
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.head.take().map(|head| {
            let mut item = self.map.remove(&head).unwrap();
            let elem = item.elem.take();
            self.head = item.next;
            if self.head.is_none() {
                // empty
                self.tail = None;
            }
            elem.unwrap()
        })
    }

    // utils - assuming item is not None
    pub fn get_next(&self, cursor: &Cursor) -> Cursor {
        self.map.get(cursor.as_ref().unwrap()).unwrap().next
    }

    pub fn get(&self, cursor: &Cursor) -> &T {
        self.map
            .get(cursor.as_ref().unwrap())
            .unwrap()
            .elem
            .as_ref()
            .unwrap()
    }

    pub fn set(&mut self, cursor: &Cursor, new_val: T) {
        self.map
            .get_mut(cursor.as_ref().unwrap())
            .unwrap()
            .elem
            .replace(new_val);
    }

    pub fn take(&mut self, cursor: &Cursor) -> T {
        self.map
            .get_mut(cursor.as_ref().unwrap())
            .unwrap()
            .elem
            .take()
            .unwrap()
    }

    pub fn exist(&self, cursor: &Cursor) -> bool {
        cursor.map_or(false, |key| self.map.contains_key(&key))
    }

    /// find_elem returns the first item non-prior to `cursor` that compare(item) is true
    /// if no such item exists, the function returns None
    pub fn find_elem_from<F: Fn(&T) -> bool>(&self, cursor: Cursor, compare: F) -> Cursor {
        let mut current = cursor;
        if !self.exist(&cursor) {
            return None;
        }
        while current.is_some() {
            if compare(self.get(&current)) {
                return current;
            }
            current = self.get_next(&current);
        }
        None
    }

    /// we make sure that the element found by the key is after `cursor`
    /// if `cursor` is None, this function returns None (same as find_elem)
    pub fn find_elem_by_key(&self, cursor: Cursor, key: HashValue) -> Cursor {
        let cursor_order = self.map.get(cursor.as_ref()?).unwrap().index;
        let item = self.map.get(&key)?;
        if item.index >= cursor_order {
            Some(key)
        } else {
            None
        }
    }
}

// tests
#[cfg(test)]
mod test {
    use super::Buffer;
    use crate::experimental::hashable::Hashable;
    use diem_crypto::HashValue;
    use std::fmt::{Debug, Formatter};

    #[derive(PartialEq, Eq)]
    pub struct HashWrapper {
        inner: HashValue,
    }

    impl Debug for HashWrapper {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "HashWrapper: [{}]", self.inner)
        }
    }

    impl From<u64> for HashWrapper {
        fn from(val: u64) -> Self {
            Self {
                inner: HashValue::from_u64(val),
            }
        }
    }

    impl Hashable for HashWrapper {
        fn hash(&self) -> HashValue {
            self.inner
        }
    }

    #[test]
    fn basics() {
        let mut buffer = Buffer::<HashWrapper>::new();

        // Check empty list behaves right
        assert_eq!(buffer.pop_front(), None);

        // Populate list
        buffer.push_back(HashWrapper::from(1));
        buffer.push_back(HashWrapper::from(2));
        buffer.push_back(HashWrapper::from(3));

        // Check normal removal
        assert_eq!(buffer.pop_front(), Some(HashWrapper::from(1)));
        assert_eq!(buffer.pop_front(), Some(HashWrapper::from(2)));

        // Push some more just to make sure nothing's corrupted
        buffer.push_back(HashWrapper::from(4));
        buffer.push_back(HashWrapper::from(5));

        // Check normal removal
        assert_eq!(buffer.pop_front(), Some(HashWrapper::from(3)));
        assert_eq!(buffer.pop_front(), Some(HashWrapper::from(4)));

        // Check exhaustion
        assert_eq!(buffer.pop_front(), Some(HashWrapper::from(5)));
        assert_eq!(buffer.pop_front(), None);
    }

    #[test]
    fn find() {
        let mut buffer = Buffer::<HashWrapper>::new();
        buffer.push_back(HashWrapper::from(1));
        buffer.push_back(HashWrapper::from(2));
        buffer.push_back(HashWrapper::from(3));

        // look for 1 (succeed)
        let res_1 = buffer.find_elem_by_key(*buffer.head_cursor(), HashValue::from_u64(1));
        assert_eq!(buffer.get(&res_1), &HashWrapper::from(1));
        // look for 4 (fail)
        let res_no_4 = buffer.find_elem_by_key(*buffer.head_cursor(), HashValue::from_u64(4));
        assert!(res_no_4.is_none());
        // look for 1 after (or on) the tail (fail)
        let res_no_1 = buffer.find_elem_by_key(*buffer.tail_cursor(), HashValue::from_u64(1));
        assert!(res_no_1.is_none());

        // look for 2 (succeed)
        let res_2 =
            buffer.find_elem_from(*buffer.head_cursor(), |item| item == &HashWrapper::from(2));
        assert_eq!(buffer.get(&res_2), &HashWrapper::from(2));
        // look for 5 (fail)
        let res_no_5 =
            buffer.find_elem_from(*buffer.head_cursor(), |item| item == &HashWrapper::from(5));
        assert!(res_no_5.is_none());
        // look for 2 after (or on) the tail (fail)
        let res_no_2 =
            buffer.find_elem_from(*buffer.tail_cursor(), |item| item == &HashWrapper::from(2));
        assert!(res_no_2.is_none());
    }

    #[test]
    fn get_set_take() {
        let mut buffer = Buffer::<HashWrapper>::new();
        buffer.push_back(HashWrapper::from(1));
        buffer.push_back(HashWrapper::from(2));
        buffer.push_back(HashWrapper::from(3));

        // test get
        assert_eq!(buffer.get(buffer.head_cursor()), &HashWrapper::from(1));
        assert_eq!(buffer.get(buffer.tail_cursor()), &HashWrapper::from(3));

        // test set
        let tail = *buffer.tail_cursor();
        buffer.set(&tail, HashWrapper::from(5));
        assert_eq!(buffer.get(buffer.tail_cursor()), &HashWrapper::from(5));

        // test take
        let head = *buffer.head_cursor();
        assert_eq!(buffer.take(&head), HashWrapper::from(1));
    }
}
